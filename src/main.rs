use chip8::Emulator;
use chip8::MinifbWindow;
use chip8::TerminalWindow;
use std::{env, fs, process::exit};

const EX_USAGE: i32 = 64;

fn main() {
    let args: Vec<String> = env::args().collect();

    // args[0] is the path to the program.
    if args.len() != 1 + 1 {
        println!("Error: Path to rom needs to be passed");
        exit(EX_USAGE);
    }

    let rom_path = &args[1];

    let rom = fs::read(rom_path).unwrap();

    let mut emulator = Emulator::new(TerminalWindow::new(), false);
    emulator.load_rom(rom);

    emulator.start();
}
