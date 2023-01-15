mod constant;
mod keypad;
mod opcode;
mod screen;

use crate::keypad::{key_to_keypad, keypad_to_key};
use crate::opcode::Opcode;
use crate::screen::Screen;
use constant::*;
use minifb::{Key, Scale, Window, WindowOptions};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{env, fs, time::Instant};

fn byte_to_bits(byte: u8) -> Vec<bool> {
    return (0..u8::BITS).rev().map(|i| byte >> i & 1 == 1).collect();
}

pub fn run() {
    let args: Vec<String> = env::args().collect();

    let rom_path = &args[1];

    let rom = fs::read(rom_path).unwrap();

    let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

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
