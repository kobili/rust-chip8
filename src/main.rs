mod chip8;
mod constants;

use chip8::Chip8;


fn main() {
    let _c8 = Chip8::new("./examples/maze.ch8");
}
