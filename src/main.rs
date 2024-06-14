mod chip8;
mod constants;

use chip8::Chip8;


#[allow(arithmetic_overflow)]
fn main() {
    let _c8 = Chip8::new("./examples/maze.ch8");

    let a = 0xFF_u8;
    let b = 0x2_u8;

    println!("0x{:x} + 0x{:x} == 0x{:x}", a, b, a.wrapping_add(b));
    println!("0x{:x} - 0x{:x} == 0x{:x}", b, a, b.wrapping_add(a));
}
