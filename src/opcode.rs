#[derive(Debug)]
pub enum Opcode {
    /// 0NNN
    Sys { nnn: u16 },
    /// 00E0
    Clear,
    /// 00EE
    Return,
    /// 1NNN
    Jump { nnn: u16 },
    /// 2NNN
    Call { nnn: u16 },
    /// 3XNN
    SkipEqualsConstant { vx: u8, nn: u8 },
    /// 4XNN
    SkipNotEqualsConstant { vx: u8, nn: u8 },
    /// 5XY0
    SkipEquals { vx: u8, vy: u8 },
    /// 6XNN
    Load { vx: u8, nn: u8 },
    /// 7XNN
    AddConstant { vx: u8, nn: u8 },
    /// 8XY0
    LoadRegister { vx: u8, vy: u8 },
    /// 8XY1
    Or { vx: u8, vy: u8 },
    /// 8XY2
    And { vx: u8, vy: u8 },
    /// 8XY3
    Xor { vx: u8, vy: u8 },
    /// 8XY4
    Add { vx: u8, vy: u8 },
    /// 8XY5
    Sub { vx: u8, vy: u8 },
    /// 8XY6
    ShiftRight { vx: u8 },
    /// 8XY7
    Subn { vx: u8, vy: u8 },
    /// 8XYE
    ShiftLeft { vx: u8 },
    /// 9XY0
    SkipNotEquals { vx: u8, vy: u8 },
    /// ANNN
    LoadIndex { nnn: u16 },
    /// BNNN
    JumpPlusV0 { nnn: u16 },
    /// CXNN
    Random { vx: u8, nn: u8 },
    /// DXYN
    Draw { vx: u8, vy: u8, n: u8 },

    /// EX9E
    KeyPressSkip { vx: u8 },
    /// EXA1
    KeyNotPressSkip { vx: u8 },

    /// FX07
    DelayTimerLoadFrom { vx: u8 },

    /// FX0A
    KeyLoad { vx: u8 },

    /// FX15
    DelayTimerLoadInto { vx: u8 },

    /// FX18
    SoundLoad { vx: u8 },

    /// FX1E
    AddIndex { vx: u8 },

    /// FX29
    LocateSprite { vx: u8 },

    /// FX33
    LoadBcd { vx: u8 },

    /// FX55
    StoreRegisters { vx: u8 },

    /// FX65
    ReadRegisters { vx: u8 },
}

impl Opcode {
    pub fn from(buf: [u8; 2]) -> Self {
        let num = u16::from_be_bytes(buf);

        // Don't polute the namespace, and let match arms below always read from
        // the matched pattern.
        match {
            // Nibbles from `num` (from most significant to least).
            let ins: u8 = ((num >> 16 / 4 * 3) & 0xF) as u8;
            let x: u8 = ((num >> 16 / 4 * 2) & 0xF) as u8;
            let y: u8 = ((num >> 16 / 4 * 1) & 0xF) as u8;
            let n: u8 = ((num >> 16 / 4 * 0) & 0xF) as u8;

            // Lower byte.
            let nn: u8 = (num & 0xFF) as u8;

            // 2nd to last nibble.
            let nnn: u16 = num << 4 >> 4;

            (ins, x, y, n, nn, nnn)
        } {
            (0x0, 0, 0xE, 0, _, _) => Opcode::Clear,
            (0x0, 0, 0xE, 0xE, _, _) => Opcode::Return,
            (0x0, _, _, _, _, nnn) => Opcode::Sys { nnn },
            (0x1, _, _, _, _, nnn) => Opcode::Jump { nnn },
            (0x2, _, _, _, _, nnn) => Opcode::Call { nnn },
            (0x3, vx, _, _, nn, _) => Opcode::SkipEqualsConstant { vx, nn },
            (0x4, vx, _, _, nn, _) => Opcode::SkipNotEqualsConstant { vx, nn },
            (0x5, vx, vy, 0, _, _) => Opcode::SkipEquals { vx, vy },
            (0x6, vx, _, _, nn, _) => Opcode::Load { vx, nn },
            (0x7, vx, _, _, nn, _) => Opcode::AddConstant { vx, nn },
            (0x8, vx, vy, 0, _, _) => Opcode::LoadRegister { vx, vy },
            (0x8, vx, vy, 1, _, _) => Opcode::Or { vx, vy },
            (0x8, vx, vy, 2, _, _) => Opcode::And { vx, vy },
            (0x8, vx, vy, 3, _, _) => Opcode::Xor { vx, vy },
            (0x8, vx, vy, 4, _, _) => Opcode::Add { vx, vy },
            (0x8, vx, vy, 5, _, _) => Opcode::Sub { vx, vy },
            (0x8, vx, _, 6, _, _) => Opcode::ShiftRight { vx },
            (0x8, vx, vy, 7, _, _) => Opcode::Subn { vx, vy },
            (0x8, vx, _, 0xE, _, _) => Opcode::ShiftLeft { vx },
            (0x9, vx, vy, 0, _, _) => Opcode::SkipNotEquals { vx, vy },
            (0xA, _, _, _, _, nnn) => Opcode::LoadIndex { nnn },
            (0xB, _, _, _, _, nnn) => Opcode::JumpPlusV0 { nnn },
            (0xC, vx, _, _, nn, _) => Opcode::Random { vx, nn },
            (0xD, vx, vy, n, _, _) => Opcode::Draw { vx, vy, n },
            (0xE, vx, 9, 0xE, _, _) => Opcode::KeyPressSkip { vx },
            (0xE, vx, 0xA, 1, _, _) => Opcode::KeyNotPressSkip { vx },
            (0xF, vx, 0, 7, _, _) => Opcode::DelayTimerLoadFrom { vx },
            (0xF, vx, 0, 0xA, _, _) => Opcode::KeyLoad { vx },
            (0xF, vx, 1, 5, _, _) => Opcode::DelayTimerLoadInto { vx },
            (0xF, vx, 1, 8, _, _) => Opcode::SoundLoad { vx },
            (0xF, vx, 1, 0xE, _, _) => Opcode::AddIndex { vx },
            (0xF, vx, 2, 9, _, _) => Opcode::LocateSprite { vx },
            (0xF, vx, 3, 3, _, _) => Opcode::LoadBcd { vx },
            (0xF, vx, 5, 5, _, _) => Opcode::StoreRegisters { vx },
            (0xF, vx, 6, 5, _, _) => Opcode::ReadRegisters { vx },

            _ => unimplemented!("Cannot parse `{num:02X?}` into opcode"),
        }
    }
}
