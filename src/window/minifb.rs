//! Implementation of [crate::window::Window] using [minifb].
//!
//! This should be the only file in this crate which depends on [minifb].

use std::time::Duration;

use crate::constant::{FPS, HEIGHT, WIDTH};
use crate::error::Error;
use crate::keypad::Key;
use minifb::{KeyRepeat, Scale, Window, WindowOptions};

pub struct MinifbWindow(Window);

impl MinifbWindow {
    pub fn new() -> Self {
        let mut window = Window::new(
            "chip8 - Press ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X8,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        // This setting affects how long `window.update` will take to return.
        window.limit_update_rate(Some(Duration::from_secs_f64(FPS)));

        Self(window)
    }
}

const PIXEL_COLOR: u32 = u32::MAX;

impl crate::window::Window for MinifbWindow {
    fn is_running(&mut self) -> bool {
        self.0.is_open() && !self.0.is_key_down(minifb::Key::Escape)
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.0.is_key_down(key.into())
    }

    fn is_key_up(&self, key: Key) -> bool {
        self.0.is_key_released(key.into())
    }

    fn wait_for_next_key(&self) -> Option<Key> {
        self.0
            .get_keys_pressed(KeyRepeat::No)
            .into_iter()
            .map(|key| Key::try_from(key))
            .filter_map(|key| key.ok())
            .nth(0)
    }

    fn update(&mut self, buffer: &[bool; WIDTH * HEIGHT]) -> Result<(), Error> {
        let buffer = buffer
            .into_iter()
            .map(|on| if *on { PIXEL_COLOR } else { 0 })
            .collect::<Vec<u32>>();

        self.0
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .map_err(|err| Error::ErrorStr(err.to_string()))
    }
}

impl TryFrom<minifb::Key> for Key {
    type Error = ();

    fn try_from(key: minifb::Key) -> Result<Self, Self::Error> {
        match key {
            minifb::Key::Key1 => Ok(Key::Key1),
            minifb::Key::Key2 => Ok(Key::Key2),
            minifb::Key::Key3 => Ok(Key::Key3),
            minifb::Key::Key4 => Ok(Key::Key4),
            minifb::Key::Q => Ok(Key::Q),
            minifb::Key::W => Ok(Key::W),
            minifb::Key::E => Ok(Key::E),
            minifb::Key::R => Ok(Key::R),
            minifb::Key::A => Ok(Key::A),
            minifb::Key::S => Ok(Key::S),
            minifb::Key::D => Ok(Key::D),
            minifb::Key::F => Ok(Key::F),
            minifb::Key::Z => Ok(Key::Z),
            minifb::Key::X => Ok(Key::X),
            minifb::Key::C => Ok(Key::C),
            minifb::Key::V => Ok(Key::V),
            _ => Err(()),
        }
    }
}

impl From<Key> for minifb::Key {
    fn from(key: Key) -> minifb::Key {
        match key {
            Key::Key1 => minifb::Key::Key1,
            Key::Key2 => minifb::Key::Key2,
            Key::Key3 => minifb::Key::Key3,
            Key::Key4 => minifb::Key::Key4,
            Key::Q => minifb::Key::Q,
            Key::W => minifb::Key::W,
            Key::E => minifb::Key::E,
            Key::R => minifb::Key::R,
            Key::A => minifb::Key::A,
            Key::S => minifb::Key::S,
            Key::D => minifb::Key::D,
            Key::F => minifb::Key::F,
            Key::Z => minifb::Key::Z,
            Key::X => minifb::Key::X,
            Key::C => minifb::Key::C,
            Key::V => minifb::Key::V,
        }
    }
}
