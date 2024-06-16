mod chip8;
mod constants;
mod utils;

use chip8::Chip8;


#[allow(arithmetic_overflow)]
fn main() {
    let _c8 = Chip8::new("./examples/maze.ch8");

    let opcode = 0xA222;
    println!("{:x}", (opcode & 0xF000) >> 12);
}
