use crate::constant::*;
use crate::keypad::Key;
use crate::opcode::Opcode;
use crate::screen::Screen;
use crate::window::Window;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;

pub struct Emulator<T: Window> {
    memory: [u8; MEMORY_SIZE],
    verbose: bool,
    registers: [u8; 16],

    pc: u16,
    index: u16,
    stack: Vec<u16>,

    delay_timer: u8,
    sound_timer: u8,

    rng: StdRng,

    screen: Screen,
    window: T,

    last_ins_time: Instant,
}

impl<T: Window> Emulator<T> {
    pub fn new(window: T, verbose: bool) -> Self {
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        FONTS
            .into_iter()
            .enumerate()
            .for_each(|(i, char)| memory[FONT_OFFSET + i] = char);

        Self {
            memory,
            screen: Screen::new(),
            verbose,
            // max pc is actually u12 (from nnn which is 12 bytes).
            pc: ROM_LOAD_OFFSET as u16,
            index: 0,
            stack: vec![],

            delay_timer: u8::MAX,
            sound_timer: u8::MAX,

            registers: [0; 16],

            rng: StdRng::seed_from_u64(1),
            window,

            last_ins_time: Instant::now(),
        }
    }
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.memory[ROM_LOAD_OFFSET..(ROM_LOAD_OFFSET + rom.len())].copy_from_slice(rom.as_slice());
    }

    pub fn start(&mut self) {
        while self.window.is_running() {
            let opcode = Opcode::from([
                self.memory[self.pc as usize],
                self.memory[self.pc as usize + 1],
            ]);

            if self.verbose {
                let current_ins_time = Instant::now();
                self.last_ins_time = current_ins_time;
                let time_since_last_ins = current_ins_time.duration_since(self.last_ins_time);
                println!(
                    "[+{}ms] pc {} {opcode:?}",
                    time_since_last_ins.as_millis(),
                    self.pc,
                );
            }

            self.pc += 2;

            match opcode {
                Opcode::Sys { nnn } => unimplemented!("SYS {nnn} is unimplemented"),
                Opcode::Clear => {
                    self.screen.clear();
                }
                Opcode::Return => {
                    let addr = self.stack.pop().unwrap();
                    self.pc = addr;
                }
                Opcode::Jump { nnn } => {
                    self.pc = nnn;
                }
                Opcode::Call { nnn } => {
                    self.stack.push(self.pc);
                    self.pc = nnn;
                }
                Opcode::SkipEqualsConstant { vx, nn } => {
                    let x = self.registers[vx as usize];
                    if x == nn {
                        self.pc += 2;
                    }
                }
                Opcode::SkipNotEqualsConstant { vx, nn } => {
                    let x = self.registers[vx as usize];
                    if x != nn {
                        self.pc += 2;
                    }
                }
                Opcode::SkipEquals { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];
                    if x == y {
                        self.pc += 2;
                    }
                }
                Opcode::Load { vx, nn } => {
                    self.registers[vx as usize] = nn;
                }
                Opcode::AddConstant { vx, nn } => {
                    let x = self.registers[vx as usize];
                    let (result, _) = x.overflowing_add(nn);
                    self.registers[vx as usize] = result;
                }
                Opcode::LoadRegister { vx, vy } => {
                    self.registers[vx as usize] = self.registers[vy as usize];
                }
                Opcode::Or { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];
                    self.registers[vx as usize] = x | y;
                }
                Opcode::And { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];
                    self.registers[vx as usize] = x & y;
                }
                Opcode::Xor { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];
                    self.registers[vx as usize] = x ^ y;
                }
                Opcode::Add { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];
                    let (result, carry) = x.overflowing_add(y);
                    self.registers[0xF] = carry as u8;
                    self.registers[vx as usize] = result;
                }
                Opcode::Sub { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];

                    let (result, overflow) = x.overflowing_sub(y);
                    self.registers[0xF] = !overflow as u8;
                    self.registers[vx as usize] = result;
                }
                Opcode::ShiftRight { vx } => {
                    let x = self.registers[vx as usize];

                    // LSB is set?
                    self.registers[0xF] = (x & 1 == 1) as u8;
                    self.registers[vx as usize] = x >> 1;
                }
                Opcode::Subn { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];

                    let (result, overflow) = y.overflowing_sub(x);
                    self.registers[0xF] = !overflow as u8;
                    self.registers[vx as usize] = result;
                }
                Opcode::ShiftLeft { vx } => {
                    let x = self.registers[vx as usize];

                    // MSB is set?
                    self.registers[0xF] = (x & (1 << (u8::BITS - 1)) > 0) as u8;
                    self.registers[vx as usize] = x << 1;
                }
                Opcode::SkipNotEquals { vx, vy } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];

                    if x != y {
                        self.pc += 2;
                    }
                }
                Opcode::LoadIndex { nnn } => {
                    self.index = nnn;
                }
                Opcode::JumpPlusV0 { nnn } => {
                    self.pc = nnn + self.registers[0] as u16;
                }
                Opcode::Random { vx, nn } => {
                    let random = self.rng.gen::<u8>();
                    self.registers[vx as usize] = random & nn;
                }
                Opcode::Draw { vx, vy, n } => {
                    let x = self.registers[vx as usize];
                    let y = self.registers[vy as usize];

                    let sprite =
                        &self.memory[self.index as usize..(self.index as usize + n as usize)];

                    let vf = &mut self.registers[0xF];
                    *vf = 0;

                    for (y_offset, row) in sprite.into_iter().enumerate() {
                        for (x_offset, bit) in byte_to_bits(*row).into_iter().enumerate() {
                            let turned_off =
                                self.screen
                                    .set(x as usize + x_offset, y as usize + y_offset, bit);
                            *vf |= turned_off as u8;
                        }
                    }
                }
                Opcode::KeyPressSkip { vx } => {
                    let keypad = self.registers[vx as usize];
                    if self.window.is_key_down(Key::from(keypad)) {
                        self.pc += 2;
                    }
                }
                Opcode::KeyNotPressSkip { vx } => {
                    let keypad = self.registers[vx as usize];
                    if self.window.is_key_up(Key::from(keypad)) {
                        self.pc += 2;
                    }
                }
                Opcode::DelayTimerLoadFrom { vx } => {
                    self.registers[vx as usize] = self.delay_timer;
                }
                Opcode::KeyLoad { vx } => {
                    match self.window.get_keys_pressed().first() {
                        // Move the PC back which should execute this instruction
                        // again.
                        None => self.pc -= 2,
                        Some(key) => self.registers[vx as usize] = key.to_keypad(),
                    }
                }
                Opcode::DelayTimerLoadInto { vx } => {
                    self.delay_timer = self.registers[vx as usize];
                }
                Opcode::SoundLoad { vx } => {
                    self.sound_timer = self.registers[vx as usize];
                }
                Opcode::AddIndex { vx } => {
                    self.index += self.registers[vx as usize] as u16;
                }
                Opcode::LocateSprite { vx } => {
                    let sprite_number = self.registers[vx as usize];
                    self.index = FONT_OFFSET as u16 + sprite_number as u16 * FONT_SIZE as u16;
                }
                Opcode::LoadBcd { vx } => {
                    let x = self.registers[vx as usize];
                    let ones = x % 10;
                    let tens = x / 10 % 10;
                    let hundreds = x / 100 % 10;

                    self.memory[self.index as usize] = hundreds;
                    self.memory[self.index as usize + 1] = tens;
                    self.memory[self.index as usize + 2] = ones;
                }
                Opcode::StoreRegisters { vx } => {
                    let index = self.index as usize;
                    let vx = vx as usize;
                    let upper = index + vx;

                    self.memory[index..=upper].copy_from_slice(&self.registers[..=vx]);
                }
                Opcode::ReadRegisters { vx } => {
                    let index = self.index as usize;
                    let vx = vx as usize;
                    let upper = index + vx;

                    self.registers[..=vx].copy_from_slice(&self.memory[index..=upper]);
                }
            };

            self.window.update(self.screen.framebuffer()).unwrap();

            self.delay_timer -= if self.delay_timer > 0 { 1 } else { 0 };
            self.sound_timer -= if self.sound_timer > 0 { 1 } else { 0 };
        }
    }
}

fn byte_to_bits(byte: u8) -> Vec<bool> {
    return (0..u8::BITS).rev().map(|i| byte >> i & 1 == 1).collect();
}
