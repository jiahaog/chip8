use crate::constant::*;

pub struct Screen([bool; WIDTH * HEIGHT]);

impl Screen {
    pub fn new() -> Self {
        Self([false; WIDTH * HEIGHT])
    }

    /// Sets the pixel value.
    ///
    /// Returns whether the pixel value at the current location was turned off.
    pub fn set(&mut self, x: usize, y: usize, bit: bool) -> bool {
        let x = x % WIDTH;
        let y = y % HEIGHT;

        let i = y * WIDTH + x;

        let prev = self.0[i];

        // a b result  turned_off
        // 1 1 0       1
        // 1 0 1       0
        // 0 1 1       0
        // 0 0 0       0

        self.0[i] = prev ^ bit;

        prev & bit
    }

    pub fn clear(&mut self) {
        for x in &mut self.0 {
            *x = false;
        }
    }

    pub fn framebuffer(&self) -> &[bool; WIDTH * HEIGHT] {
        &self.0
    }
}
