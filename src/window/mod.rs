use crate::error::Error;
use crate::keypad::Key;

pub mod minifb;

/// Interface for UI backends.
pub trait Window {
    fn is_running(&self) -> bool;

    fn is_key_down(&self, key: Key) -> bool;

    fn is_key_up(&self, key: Key) -> bool;

    fn get_keys_pressed(&self) -> Vec<Key>;

    fn update(&mut self, buffer: &Vec<bool>) -> Result<(), Error>;
}
