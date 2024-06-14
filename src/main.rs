mod chip8;

use chip8::Chip8;


fn main() {
    let mut c8 = Chip8::new();
    c8.load_instructions_from_file("./examples/maze.ch8");
}
