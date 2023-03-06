# chip8

Basic emulator for [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8).

Supports output to the current terminal, or alternatively to a separate window with the `--renderer=window` option.

(Tested only on Linux, but in theory the code should be cross-platform)

## Usage

With the Rust toolchain [installed](https://www.rust-lang.org/tools/install):

```sh
cargo run --quiet -- --rom $PATH_TO_ROM
```

### Example

[![asciicast of running roms/maze.ch8](https://asciinema.org/a/NFhBwTN7Ee7WT0JyRIhEuY6fg.svg)](https://asciinema.org/a/NFhBwTN7Ee7WT0JyRIhEuY6fg)

## Controls

Keys are mapped from the following on your keyboard:

```
1	2	3	4
Q	W	E	R
A	S	D	F
Z	X	C	V
```

to the hex keypad keys expected by CHIP-8 programs:

```
1	2	3	C
4	5	6	D
7	8	9	E
A	0	B	F
```

## Missing Pieces

- Tests
- Things like timings / inputs might be weird
- Sound

## References

- [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator)
- [Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)
- [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
