use crate::constant::{FPS, HEIGHT, WIDTH};
use crate::error::Error;
use crate::keypad::Key;
use crate::window::Window;

pub struct MinifbWindow(minifb::Window);

impl MinifbWindow {
    pub fn new() -> Self {
        let mut window = minifb::Window::new(
            "chip8 - Press ESC to exit",
            WIDTH,
            HEIGHT,
            minifb::WindowOptions {
                scale: minifb::Scale::X8,
                ..minifb::WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        // This setting affects how long `window.update` will take to return.
        window.limit_update_rate(Some(std::time::Duration::from_secs_f64(FPS)));

        Self(window)
    }
}

const PIXEL_COLOR: u32 = u32::MAX;

impl Window for MinifbWindow {
    fn is_running(&self) -> bool {
        self.0.is_open() && !self.0.is_key_down(minifb::Key::Escape)
    }

    fn is_key_down(&self, key: Key) -> bool {
        self.0.is_key_down(key.into())
    }

    fn is_key_up(&self, key: Key) -> bool {
        self.0.is_key_released(key.into())
    }

    fn get_keys_pressed(&self) -> Vec<Key> {
        self.0
            .get_keys_pressed(minifb::KeyRepeat::No)
            .into_iter()
            .map(|key| key.into())
            .collect()
    }

    fn update(&mut self, buffer: &Vec<bool>) -> Result<(), Error> {
        let buffer = buffer
            .into_iter()
            .map(|on| if *on { PIXEL_COLOR } else { 0 })
            .collect::<Vec<u32>>();

        self.0
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .map_err(|err| Error::ErrorStr(err.to_string()))
    }
}

impl From<minifb::Key> for Key {
    fn from(key: minifb::Key) -> Self {
        match key {
            minifb::Key::Key1 => Key::Key1,
            minifb::Key::Key2 => Key::Key2,
            minifb::Key::Key3 => Key::Key3,
            minifb::Key::Key4 => Key::Key4,
            minifb::Key::Q => Key::Q,
            minifb::Key::W => Key::W,
            minifb::Key::E => Key::E,
            minifb::Key::R => Key::R,
            minifb::Key::A => Key::A,
            minifb::Key::S => Key::S,
            minifb::Key::D => Key::D,
            minifb::Key::F => Key::F,
            minifb::Key::Z => Key::Z,
            minifb::Key::X => Key::X,
            minifb::Key::C => Key::C,
            minifb::Key::V => Key::V,
            x => panic!("Unknown key {:?}", x),
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
