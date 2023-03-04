//! Implementation of [crate::window::Window], rendering to the terminal output.
//!
//! This should be the only file in this crate which depends on [crossterm].

use std::io::stdout;
use std::io::Stdout;
use std::io::Write;
use std::iter::zip;
use std::thread;
use std::time::Duration;

use crate::constant::{FPS, HEIGHT, WIDTH};
use crate::error::Error;
use crate::keypad::Key;
use crate::window::Window;
use crossterm::cursor;
use crossterm::event::read;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::style;
use crossterm::terminal;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ExecutableCommand;
use crossterm::QueueableCommand;

pub struct TerminalWindow {
    stdout: Stdout,
    lines: Vec<String>,
}

impl TerminalWindow {
    pub fn new() -> Self {
        let mut stdout = stdout();

        // This causes the terminal to be output on an alternate buffer.
        stdout.execute(EnterAlternateScreen).unwrap();
        stdout.execute(crossterm::cursor::Hide).unwrap();

        terminal::enable_raw_mode().unwrap();

        Self {
            stdout,
            lines: vec!["".to_string(); HEIGHT / 2],
        }
    }
}

const FAST_IO_DURATION: Duration = Duration::from_secs((FPS / 10.) as u64);

impl Window for TerminalWindow {
    fn is_running(&mut self) -> bool {
        // When `enable_raw_mode` is set, `Ctrl-C` events to interrupt the
        // process is ignored. So manually handle the interrupt.

        if !crossterm::event::poll(FAST_IO_DURATION).unwrap() {
            return true;
        }
        // Guaranteed not to block if `poll` above is true.
        let event = crossterm::event::read().unwrap();

        if event == Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)) {
            // Be a good citizen and restore the previous terminal.
            terminal::disable_raw_mode().unwrap();
            self.stdout.execute(LeaveAlternateScreen).unwrap();
            false
        } else {
            true
        }
    }

    fn is_key_down(&self, key: Key) -> bool {
        match read().unwrap() {
            crossterm::event::Event::Key(received) => key == received.into(),
            _ => false,
        }
    }

    fn is_key_up(&self, key: Key) -> bool {
        // This doesn't exactly match the semantics of the method, but I guess
        // it's good enough for now.
        self.wait_for_next_key()
            .map_or(true, |received| key != received)
    }

    fn wait_for_next_key(&self) -> Option<Key> {
        match read().unwrap() {
            crossterm::event::Event::Key(received) => Some(received.into()),
            _ => None,
        }
    }

    fn update(&mut self, buffer: &[bool; WIDTH * HEIGHT]) -> Result<(), Error> {
        let mut buffer = buffer.to_vec();
        // Always process an even number of rows.
        if buffer.len() % 2 != 0 {
            buffer.extend(vec![false; WIDTH]);
        }

        // Each element is one pixel, but when it is rendered to the terminal,
        // two rows share one character, using the unicode BLOCK characters.

        // Group alternate rows together, so zipping them allows two consecutive
        // rows to be processed into terminal characters at the same time.
        let tops = buffer
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let row = i / WIDTH;

                if row % 2 == 0 {
                    true
                } else {
                    false
                }
            })
            .map(|(_, val)| *val);
        let bottoms = buffer
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let row = i / WIDTH;

                if row % 2 == 1 {
                    true
                } else {
                    false
                }
            })
            .map(|(_, val)| *val);

        let lines = zip(tops, bottoms)
            .enumerate()
            .fold(vec![], |mut acc, (i, top_and_bottom)| {
                if i % WIDTH == 0 {
                    acc.push("".to_string());
                }
                let character = match top_and_bottom {
                    (true, true) => BLOCK_FULL,
                    (true, false) => BLOCK_UPPER,
                    (false, true) => BLOCK_LOWER,
                    (false, false) => BLOCK_EMPTY,
                };

                let current_line = acc.last_mut().unwrap();
                current_line.push(character);

                acc
            });

        assert!(lines.len() == HEIGHT / 2);
        assert!(lines.len() == self.lines.len());

        // Refreshing the entire terminal (with the clear char) and outputting
        // everything on every iteration is costly and causes the terminal to
        // flicker.
        //
        // Instead, only "re-render" the current line, if it is different from
        // the previous frame.

        for (i, (prev, current)) in zip(&self.lines, &lines).enumerate() {
            if prev != current {
                self.stdout.queue(cursor::MoveTo(0, i as u16))?;
                self.stdout.queue(Clear(ClearType::CurrentLine))?;
                self.stdout.queue(style::Print(current))?;
            }
        }

        self.stdout.flush()?;

        self.lines = lines;

        thread::sleep(Duration::from_secs_f64(FPS));

        Ok(())
    }
}

impl From<crossterm::event::KeyEvent> for Key {
    fn from(key: crossterm::event::KeyEvent) -> Self {
        use crossterm::event::KeyCode::*;
        match key.code {
            Char('1') => Key::Key1,
            Char('2') => Key::Key2,
            Char('3') => Key::Key3,
            Char('4') => Key::Key4,
            Char('q') => Key::Q,
            Char('w') => Key::W,
            Char('e') => Key::E,
            Char('r') => Key::R,
            Char('a') => Key::A,
            Char('s') => Key::S,
            Char('d') => Key::D,
            Char('f') => Key::F,
            Char('z') => Key::Z,
            Char('x') => Key::X,
            Char('c') => Key::C,
            Char('v') => Key::V,
            x => panic!("Unknown key {:?}", x),
        }
    }
}

const BLOCK_LOWER: char = '▄';
const BLOCK_UPPER: char = '▀';
const BLOCK_FULL: char = '█';
const BLOCK_EMPTY: char = ' ';

impl From<crossterm::ErrorKind> for Error {
    fn from(value: crossterm::ErrorKind) -> Self {
        Error::ErrorStr(value.to_string())
    }
}
