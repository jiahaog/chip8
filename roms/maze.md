# Maze, by David Winter

[Source](https://github.com/dmatlack/chip8/blob/aebb1ae08505d129e56ae61ee08d3193a29a2e1a/roms/demos/Maze%20%5BDavid%20Winter%2C%20199x%5D.ch8)

Drawing a random maze like this one consists in drawing random diagonal
lines. There are two possibilities: right-to-left line, and left-to-right
line. Each line is composed of a 4*4 bitmap. As the lines must form non-
circular angles, the two bitmaps won't be '/' and '\'. The first one
(right line) will be a little bit modified. See at the end of this source.

The maze is composed of 16 lines (as the bitmaps are 4 pixels high), each
line consists of 32 bitmaps.
Bitmaps are drawn in random mode. We choose a random value (0 or 1).
If it is 1, we draw a left line bitmap. If it is 0, we draw a right one.
