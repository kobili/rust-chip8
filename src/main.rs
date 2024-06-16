mod chip8;
mod constants;
mod utils;

use chip8::Chip8;


#[allow(arithmetic_overflow)]
fn main() {
    let mut c8 = Chip8::new("./examples/maze.ch8");
    c8.execute_opcode(0xfa18);

    let opcode = 0xA222;
    println!("{:x}", (opcode & 0xF000) >> 12);
}
