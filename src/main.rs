mod chip8;
mod constants;

use chip8::Chip8;


#[allow(arithmetic_overflow)]
fn main() {
    let _c8 = Chip8::new("./examples/maze.ch8");

    let a = 0xFFu8;
    let b = 0x1u8;

    println!("0x{:x} - 0x{:x} == 0x{:x}", a, b, a.wrapping_sub(b));
}
