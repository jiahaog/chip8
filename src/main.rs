extern crate minifb;

use std::{env, fs, time::Instant};

use minifb::{Key, Scale, Window, WindowOptions};
use rand::{rngs::StdRng, Rng, SeedableRng};

const WIDTH: usize = 64;

const HEIGHT: usize = 32;

/// Offset in memory to place fonts.
///
/// For some reason, it is popular to put the font from `050 - 09F`.
const FONT_OFFSET: usize = 050;

const FONT_SIZE: usize = 5;

const FONTS: [u8; FONT_SIZE * 16] = [
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

const PIXEL_COLOR: u32 = from_u8_rgb(0, 0, 255);

const FPS: f64 = 1. / 60.;

struct Screen(Vec<u32>);

impl Screen {
    fn new() -> Self {
        Self(vec![0; WIDTH * HEIGHT])
    }

    /// Sets the pixel value.
    ///
    /// Returns whether the pixel value at the current location was turned off.
    fn set(&mut self, x: usize, y: usize, bit: bool) -> bool {
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

    fn clear(&mut self) {
        for x in &mut self.0 {
            *x = 0;
        }
    }

    fn framebuffer(&self) -> &Vec<u32> {
        &self.0
    }
}

fn byte_to_bits(byte: u8) -> Vec<bool> {
    return (0..u8::BITS).rev().map(|i| byte >> i & 1 == 1).collect();
}

/// Maps the following keypad keys to a Key.
///
/// Expect keypad to be one of the following:
/// ```
/// 1 2 3 C
/// 4 5 6 D
/// 7 8 9 E
/// A 0 B F
/// ```
///
/// Which maps to a grid of keys on the keyboard, from number 1 on the top left
/// to V on the bottom right.
fn keypad_to_key(keypad: u8) -> Key {
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
        _ => panic!("Unknown key: {keypad}"),
    }
}

/// Inversion of `keypad_to_key`.
fn key_to_keypad(key: &Key) -> u8 {
    match key {
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
        _ => panic!("Unknown key: {key:?}"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_path = &args[1];

    let rom = fs::read(rom_path).unwrap();

    let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    // copy_into_array(&mut memory, rom.as_slice(), ROM_LOAD_OFFSET);

    memory[ROM_LOAD_OFFSET..(ROM_LOAD_OFFSET + rom.len())].copy_from_slice(rom.as_slice());

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

    let mut rng = StdRng::seed_from_u64(1);

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

    // This setting affects how long `window.update` will take to return.
    window.limit_update_rate(Some(std::time::Duration::from_secs_f64(FPS)));

    let mut last_ins_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_ins_time = Instant::now();
        let time_since_last_ins = current_ins_time.duration_since(last_ins_time);
        last_ins_time = current_ins_time;

        let opcode = Opcode::from([memory[pc as usize], memory[pc as usize + 1]]);
        println!(
            "[+{}ms] pc {pc} {opcode:?}",
            time_since_last_ins.as_millis()
        );

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
            Opcode::SkipEqualsConstant { vx, nn } => {
                let x = registers[vx as usize];
                if x == nn {
                    pc += 2;
                }
            }
            Opcode::SkipNotEqualsConstant { vx, nn } => {
                let x = registers[vx as usize];
                if x != nn {
                    pc += 2;
                }
            }
            Opcode::SkipEquals { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];
                if x == y {
                    pc += 2;
                }
            }
            Opcode::Load { vx, nn } => {
                registers[vx as usize] = nn;
            }
            Opcode::AddConstant { vx, nn } => {
                let x = registers[vx as usize];
                let (result, _) = x.overflowing_add(nn);
                registers[vx as usize] = result;
            }
            Opcode::LoadRegister { vx, vy } => {
                registers[vx as usize] = registers[vy as usize];
            }
            Opcode::Or { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];
                registers[vx as usize] = x | y;
            }
            Opcode::And { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];
                registers[vx as usize] = x & y;
            }
            Opcode::Xor { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];
                registers[vx as usize] = x ^ y;
            }
            Opcode::Add { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];
                let (result, carry) = x.overflowing_add(y);
                registers[0xF] = carry as u8;
                registers[vx as usize] = result;
            }
            Opcode::Sub { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];

                let (result, overflow) = x.overflowing_sub(y);
                registers[0xF] = !overflow as u8;
                registers[vx as usize] = result;
            }
            Opcode::ShiftRight { vx } => {
                let x = registers[vx as usize];

                // LSB is set?
                registers[0xF] = (x & 1 == 1) as u8;
                registers[vx as usize] = x >> 1;
            }
            Opcode::Subn { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];

                let (result, overflow) = y.overflowing_sub(x);
                registers[0xF] = !overflow as u8;
                registers[vx as usize] = result;
            }
            Opcode::ShiftLeft { vx } => {
                let x = registers[vx as usize];

                // MSB is set?
                registers[0xF] = (x & (1 << (u8::BITS - 1)) > 0) as u8;
                registers[vx as usize] = x << 1;
            }
            Opcode::SkipNotEquals { vx, vy } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];

                if x != y {
                    pc += 2;
                }
            }
            Opcode::LoadIndex { nnn } => {
                index = nnn;
            }
            Opcode::JumpPlusV0 { nnn } => {
                pc = nnn + registers[0] as u16;
            }
            Opcode::Random { vx, nn } => {
                let random = rng.gen::<u8>();
                registers[vx as usize] = random & nn;
            }
            Opcode::Draw { vx, vy, n } => {
                let x = registers[vx as usize];
                let y = registers[vy as usize];

                let sprite = &memory[index as usize..(index as usize + n as usize)];

                let vf = &mut registers[0xF];
                *vf = 0;

                for (y_offset, row) in sprite.into_iter().enumerate() {
                    for (x_offset, bit) in byte_to_bits(*row).into_iter().enumerate() {
                        let turned_off =
                            screen.set(x as usize + x_offset, y as usize + y_offset, bit);
                        *vf |= turned_off as u8;
                    }
                }
            }
            Opcode::KeyPressSkip { vx } => {
                let keypad = registers[vx as usize];
                if window.is_key_down(keypad_to_key(keypad)) {
                    pc += 2;
                }
            }
            Opcode::KeyNotPressSkip { vx } => {
                let keypad = registers[vx as usize];
                if window.is_key_released(keypad_to_key(keypad)) {
                    pc += 2;
                }
            }
            Opcode::DelayTimerLoadFrom { vx } => {
                registers[vx as usize] = delay_timer;
            }
            Opcode::KeyLoad { vx } => {
                match window.get_keys_pressed(minifb::KeyRepeat::No).first() {
                    // Move the PC back which should execute this instruction
                    // again.
                    None => pc -= 2,
                    Some(key) => {
                        registers[vx as usize] = key_to_keypad(key);
                    }
                }
            }
            Opcode::DelayTimerLoadInto { vx } => {
                delay_timer = registers[vx as usize];
            }
            Opcode::SoundLoad { vx } => {
                sound_timer = registers[vx as usize];
            }
            Opcode::AddIndex { vx } => {
                index += registers[vx as usize] as u16;
            }
            Opcode::LocateSprite { vx } => {
                let sprite_number = registers[vx as usize];
                index = FONT_OFFSET as u16 + sprite_number as u16 * FONT_SIZE as u16;
            }
            Opcode::LoadBcd { vx } => {
                let x = registers[vx as usize];
                let ones = x % 10;
                let tens = x / 10 % 10;
                let hundreds = x / 100 % 10;

                memory[index as usize] = hundreds;
                memory[index as usize + 1] = tens;
                memory[index as usize + 2] = ones;
            }
            Opcode::StoreRegisters { vx } => {
                let index = index as usize;
                let vx = vx as usize;
                let upper = index + vx;

                memory[index..=upper].copy_from_slice(&registers[..=vx]);
            }
            Opcode::ReadRegisters { vx } => {
                let index = index as usize;
                let vx = vx as usize;
                let upper = index + vx;

                registers[..=vx].copy_from_slice(&memory[index..=upper]);
            }
        };

        window
            .update_with_buffer(screen.framebuffer(), WIDTH, HEIGHT)
            .unwrap();

        delay_timer -= if delay_timer > 0 { 1 } else { 0 };
        sound_timer -= if sound_timer > 0 { 1 } else { 0 };
    }
}

const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
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
