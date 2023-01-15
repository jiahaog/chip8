use crate::constant::*;

pub struct Screen(Vec<u32>);

impl Screen {
    pub fn new() -> Self {
        Self(vec![0; WIDTH * HEIGHT])
    }

    /// Sets the pixel value.
    ///
    /// Returns whether the pixel value at the current location was turned off.
    pub fn set(&mut self, x: usize, y: usize, bit: bool) -> bool {
        let x = x % WIDTH;
        let y = y % HEIGHT;

        let i = y * WIDTH + x;

        let prev = self.0[i] == PIXEL_COLOR;

        // a b result  turned_off
        // 1 1 0       1
        // 1 0 1       0
        // 0 1 1       0
        // 0 0 0       0

        self.0[i] = if prev ^ bit { PIXEL_COLOR } else { 0 };

        prev & bit
    }

    pub fn clear(&mut self) {
        for x in &mut self.0 {
            *x = 0;
        }
    }

    pub fn framebuffer(&self) -> &Vec<u32> {
        &self.0
    }
}
