pub enum Key {
    Key1,
    Key2,
    Key3,
    Key4,
    Q,
    W,
    E,
    R,
    A,
    S,
    D,
    F,
    Z,
    X,
    C,
    V,
}

impl Key {
    /// Maps the following keypad keys to a Key.
    ///
    /// Expect keypad to be one of the following:
    ///
    /// ```txt
    /// 1 2 3 C
    /// 4 5 6 D
    /// 7 8 9 E
    /// A 0 B F
    /// ```
    ///
    /// Which maps to a grid of keys on the keyboard, from number 1 on the top left
    /// to V on the bottom right.
    pub fn from(keypad: u8) -> Self {
        // TODO: Implement with keycodes instead.
        match keypad {
            0x1 => Key::Key1,
            0x2 => Key::Key2,
            0x3 => Key::Key3,
            0xC => Key::Key4,
            0x4 => Key::Q,
            0x5 => Key::W,
            0x6 => Key::E,
            0xD => Key::R,
            0x7 => Key::A,
            0x8 => Key::S,
            0x9 => Key::D,
            0xE => Key::F,
            0xA => Key::Z,
            0x0 => Key::X,
            0xB => Key::C,
            0xF => Key::V,
            x => panic!("Unknown keypad key '{}'", x),
        }
    }

    /// Inversion of `keypad_to_key`.
    pub fn to_keypad(&self) -> u8 {
        match self {
            Key::Key1 => 0x1,
            Key::Key2 => 0x2,
            Key::Key3 => 0x3,
            Key::Key4 => 0xC,
            Key::Q => 0x4,
            Key::W => 0x5,
            Key::E => 0x6,
            Key::R => 0xD,
            Key::A => 0x7,
            Key::S => 0x8,
            Key::D => 0x9,
            Key::F => 0xE,
            Key::Z => 0xA,
            Key::X => 0x0,
            Key::C => 0xB,
            Key::V => 0xF,
        }
    }
}
