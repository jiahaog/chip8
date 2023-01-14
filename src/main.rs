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
            Opcode::Sys(addr) => unimplemented!("SYS {addr} is unimplemented"),
            Opcode::Clear => {
                screen.clear();
            }
            Opcode::Return => {
                let addr = stack.pop().unwrap();
                pc = addr;
            }
            Opcode::Jump(nnn) => {
                pc = nnn;
            }
            Opcode::Call(addr) => {
                stack.push(pc);
                pc = addr;
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
            Opcode::Draw { x, y, n } => {
                let vx = registers[x as usize];
                let vy = registers[y as usize];

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
    Sys(u16),
    Clear,
    Return,
    Jump(u16),
    Call(u16),
    SkipEqualsConstant { vx: u8, nn: u8 },
    SkipNotEqualsConstant { vx: u8, nn: u8 },
    SkipEquals { vx: u8, vy: u8 },
    Load { vx: u8, nn: u8 },
    AddConstant { vx: u8, nn: u8 },
    LoadRegister { vx: u8, vy: u8 },
    Or { vx: u8, vy: u8 },
    And { vx: u8, vy: u8 },
    Xor { vx: u8, vy: u8 },
    Add { vx: u8, vy: u8 },
    Sub { vx: u8, vy: u8 },
    ShiftRight { vx: u8 },
    Subn { vx: u8, vy: u8 },
    ShiftLeft { vx: u8 },
    SkipNotEquals { vx: u8, vy: u8 },
    LoadIndex { nnn: u16 },
    JumpPlusV0 { nnn: u16 },
    Random { vx: u8, nn: u8 },
    Draw { x: u8, y: u8, n: u8 },

    KeyPressSkip { vx: u8 },
    KeyNotPressSkip { vx: u8 },

    DelayTimerLoadFrom { vx: u8 },

    KeyLoad { vx: u8 },

    DelayTimerLoadInto { vx: u8 },

    SoundLoad { vx: u8 },

    AddIndex { vx: u8 },

    LocateSprite { vx: u8 },

    LoadBcd { vx: u8 },

    StoreRegisters { vx: u8 },

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
            // 00E0
            (0x0, 0, 0xE, 0, _, _) => Opcode::Clear,
            // 00EE
            (0x0, 0, 0xE, 0xE, _, _) => Opcode::Return,
            // 0NNN
            (0x0, _, _, _, _, nnn) => Opcode::Sys(nnn),
            // 1NNN
            (0x1, _, _, _, _, nnn) => Opcode::Jump(nnn),
            // 2NNN
            (0x2, _, _, _, _, nnn) => Opcode::Call(nnn),
            // 6XNN
            (0x6, x, _, _, nn, _) => Opcode::Load { vx: x, nn },
            // 7XNN
            (0x7, x, _, _, nn, _) => Opcode::AddConstant { vx: x, nn },
            // ANNN
            (0xA, _, _, _, _, nnn) => Opcode::LoadIndex { nnn },
            // DXYN
            (0xD, x, y, n, _, _) => Opcode::Draw { x, y, n },
            _ => unimplemented!("{:02X?} is unimplemented", num),
        }
    }
}
