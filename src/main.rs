use chip8::Emulator;
use chip8::MinifbWindow;
use chip8::TerminalWindow;
use clap::CommandFactory;
use clap::Parser;
use std::{fs, process::exit};

const EX_USAGE: i32 = 64;

/// Emulator for CHIP-8.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the rom to load
    #[arg(long)]
    rom: String,

    /// The renderer for the UI.
    ///
    /// Either `terminal` (default) or `window` can be passed.
    #[arg(long)]
    renderer: Option<String>,

    /// Verbose mode
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let rom_path = args.rom;

    let rom = fs::read(rom_path).unwrap();

    // Seems like clap doesn't let us use ValueEnums for options so we have to
    // result to this manual parsing.
    let mut emulator = match args.renderer {
        Some(renderer) => {
            if renderer == "terminal" {
                Emulator::new(Box::new(TerminalWindow::new()), args.verbose)
            } else if renderer == "window" {
                Emulator::new(Box::new(MinifbWindow::new()), args.verbose)
            } else {
                let mut cmd = Args::command();
                cmd.print_help().unwrap();
                exit(EX_USAGE);
            }
        }
        None => Emulator::new(Box::new(TerminalWindow::new()), args.verbose),
    };

    emulator.load_rom(rom);

    emulator.start();
}
