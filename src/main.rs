extern crate minifb;

use std::{env, fs};

use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;

const HEIGHT: usize = 32;

/// Offset in memory to place fonts.
///
/// For some reason, it is popular to put the font from `050 - 09F`.
const FONT_OFFSET: usize = 050;

const FONTS: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const MEMORY_SIZE: usize = 4096;

/// Offset in memory to insert a rom.
const ROM_LOAD_OFFSET: usize = 512;

fn copy_into_array(array: &mut [u8; MEMORY_SIZE], slice: &[u8], offset: usize) {
    for (i, x) in slice.into_iter().enumerate() {
        array[offset + i] = *x;
    }
}

struct Screen([[bool; WIDTH]; HEIGHT]);

impl Screen {
    fn new() -> Self {
        Self([[false; WIDTH]; HEIGHT])
    }

    /// Sets the pixel value.
    ///
    /// Returns whether the pixel value at the current location was turned off.
    fn set(&mut self, x: u8, y: u8, flip: bool) -> bool {
        let x = x as usize % WIDTH;
        let y = y as usize % HEIGHT;

        let current = &mut self.0[y][x];
        let prev = *current;

        // a b result  turned_off
        // 1 1 0       1
        // 1 0 1       0
        // 0 1 1       0
        // 0 0 0       0

        *current ^= flip;

        prev & flip
    }

    fn to_row(&self) -> [bool; WIDTH * HEIGHT] {
        let mut result = [false; WIDTH * HEIGHT];

        let mut i = 0;
        for row in self.0 {
            for value in row {
                result[i] = value;
                i += 1;
            }
        }

        result
    }

    fn clear(&mut self) {
        self.0 = [[false; WIDTH]; HEIGHT];
    }
}

fn byte_to_bits(byte: u8) -> Vec<bool> {
    return (0..u8::BITS).rev().map(|i| byte >> i & 1 == 1).collect();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_path = &args[1];

    let rom = fs::read(rom_path).unwrap();

    let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    copy_into_array(&mut memory, rom.as_slice(), ROM_LOAD_OFFSET);

    FONTS
        .into_iter()
        .enumerate()
        .for_each(|(i, char)| memory[FONT_OFFSET + i] = char);

    let mut screen = Screen::new();
    // max pc is actually u12 (from nnn which is 12 bytes).
    let mut pc: u16 = ROM_LOAD_OFFSET as u16;

    let mut index: u16 = 0;
    let mut stack: Vec<u16> = vec![];

    let mut delay_timer: u8 = u8::MAX;
    let mut sound_timer: u8 = u8::MAX;

    let mut registers: [u8; 16] = [0; 16];

    let mut window = Window::new(
        "Test - ESC to exit",
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

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let opcode = Opcode::from([memory[pc as usize], memory[pc as usize + 1]]);
        pc += 2;

        match opcode {
            Opcode::Sys { nnn } => unimplemented!("SYS {nnn} is unimplemented"),
            Opcode::Clear => {
                screen.clear();
            }
            Opcode::Return => {
                let addr = stack.pop().unwrap();
                pc = addr;
            }
            Opcode::Jump { nnn } => {
                pc = nnn;
            }
            Opcode::Call { nnn } => {
                stack.push(pc);
                pc = nnn;
            }
            Opcode::Load { vx, nn } => {
                registers[vx as usize] = nn;
            }
            Opcode::AddConstant { vx, nn } => {
                registers[vx as usize] += nn;
            }
            Opcode::LoadIndex { nnn } => {
                index = nnn;
            }
            Opcode::Draw { vx, vy, n } => {
                let vx = registers[vx as usize];
                let vy = registers[vy as usize];

                let sprite = &memory[index as usize..(index as usize + n as usize)];

                let vf = &mut registers[0xF];
                *vf = 0;

                for (y_offset, row) in sprite.into_iter().enumerate() {
                    for (x_offset, bit) in byte_to_bits(*row).into_iter().enumerate() {
                        let turned_off = screen.set(vx + x_offset as u8, vy + y_offset as u8, bit);
                        *vf |= turned_off as u8;
                    }
                }
            }
            opcode => unimplemented!("{opcode:?} is not yet implemented"),
        };

        let framebuffer = screen
            .to_row()
            .into_iter()
            .map(|value| from_u8_rgb(0, 0, if value { 255 } else { 0 }))
            .collect::<Vec<u32>>();

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&framebuffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

#[derive(Debug)]
enum Opcode {
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
    fn from(buf: [u8; 2]) -> Self {
        let num = u16::from_be_bytes(buf);
        // Nibbles from `num` (from most significant to least).
        let ins: u8 = ((num >> 16 / 4 * 3) & 0xF) as u8;
        let x: u8 = ((num >> 16 / 4 * 2) & 0xF) as u8;
        let y: u8 = ((num >> 16 / 4 * 1) & 0xF) as u8;
        let n: u8 = ((num >> 16 / 4 * 0) & 0xF) as u8;

        // Lower byte.
        let nn: u8 = (num & 0xFF) as u8;

        // 2nd to last nibble.
        let nnn: u16 = num << 4 >> 4;

        match (ins, x, y, n, nn, nnn) {
            (0x0, 0, 0xE, 0, _, _) => Opcode::Clear,
            (0x0, 0, 0xE, 0xE, _, _) => Opcode::Return,
            (0x0, _, _, _, _, nnn) => Opcode::Sys { nnn },
            (0x1, _, _, _, _, nnn) => Opcode::Jump { nnn },
            (0x2, _, _, _, _, nnn) => Opcode::Call { nnn },
            (0x6, vx, _, _, nn, _) => Opcode::Load { vx, nn },
            (0x7, vx, _, _, nn, _) => Opcode::AddConstant { vx, nn },
            (0xA, _, _, _, _, nnn) => Opcode::LoadIndex { nnn },
            (0xD, vx, vy, n, _, _) => Opcode::Draw { vx, vy, n },
            _ => unimplemented!("{:02X?} is unimplemented", num),
        }
    }
}
