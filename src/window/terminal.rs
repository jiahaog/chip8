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
use crossterm::cursor::Hide;
use crossterm::cursor::MoveTo;
use crossterm::cursor::Show;
use crossterm::event::read;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::style::Print;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::ErrorKind;
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
        stdout.execute(Hide).unwrap();

        enable_raw_mode().unwrap();

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
            disable_raw_mode().unwrap();
            self.stdout.execute(Show).unwrap();
            self.stdout.execute(LeaveAlternateScreen).unwrap();
            false
        } else {
            true
        }
    }

    fn is_key_down(&self, key: Key) -> bool {
        match read().unwrap() {
            Event::Key(received) => {
                if let Ok(received_key) = Key::try_from(received) {
                    received_key == key
                } else {
                    false
                }
            }
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
            Event::Key(received) => {
                if let Ok(received_key) = Key::try_from(received) {
                    Some(received_key)
                } else {
                    None
                }
            }
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
                self.stdout.queue(MoveTo(0, i as u16))?;
                self.stdout.queue(Clear(ClearType::CurrentLine))?;
                self.stdout.queue(Print(current))?;
            }
        }

        self.stdout.flush()?;

        self.lines = lines;

        thread::sleep(Duration::from_secs_f64(FPS));

        Ok(())
    }
}

impl TryFrom<KeyEvent> for Key {
    type Error = ();

    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        use crossterm::event::KeyCode::*;
        match event.code {
            Char('1') => Ok(Key::Key1),
            Char('2') => Ok(Key::Key2),
            Char('3') => Ok(Key::Key3),
            Char('4') => Ok(Key::Key4),
            Char('q') => Ok(Key::Q),
            Char('w') => Ok(Key::W),
            Char('e') => Ok(Key::E),
            Char('r') => Ok(Key::R),
            Char('a') => Ok(Key::A),
            Char('s') => Ok(Key::S),
            Char('d') => Ok(Key::D),
            Char('f') => Ok(Key::F),
            Char('z') => Ok(Key::Z),
            Char('x') => Ok(Key::X),
            Char('c') => Ok(Key::C),
            Char('v') => Ok(Key::V),
            _ => Err(()),
        }
    }
}

const BLOCK_LOWER: char = '▄';
const BLOCK_UPPER: char = '▀';
const BLOCK_FULL: char = '█';
const BLOCK_EMPTY: char = ' ';

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Error::ErrorStr(value.to_string())
    }
}
