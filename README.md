# chip8

Emulator for [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8).

## Usage

With the Rust toolchain [installed](https://www.rust-lang.org/tools/install):

```sh
cargo run -- $PATH_TO_ROM
```

(tested only on Linux)

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

TODO: Use keycodes instead of string constants.

## Testing

Manually only for now.

```sh
cargo run -- roms/maze.ch8
```

This should show the following image after it is stable:

![Image showing the output of loading roms/maze.ch8](maze.png)

## References

- [Guide to making a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator)
- [Wikipedia](https://en.wikipedia.org/wiki/CHIP-8)
- [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
