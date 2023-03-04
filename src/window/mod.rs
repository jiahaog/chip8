use crate::constant::{HEIGHT, WIDTH};
use crate::error::Error;
use crate::keypad::Key;

pub mod minifb;
pub mod terminal;

/// Interface for UI backends.
pub trait Window {
    fn is_running(&mut self) -> bool;

    fn is_key_down(&self, key: Key) -> bool;

    fn is_key_up(&self, key: Key) -> bool;

    fn wait_for_next_key(&self) -> Option<Key>;

    fn update(&mut self, buffer: &[bool; WIDTH * HEIGHT]) -> Result<(), Error>;
}
