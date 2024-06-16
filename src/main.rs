mod chip8;
mod constants;
mod utils;

use chip8::Chip8;


#[allow(arithmetic_overflow)]
fn main() {
    let _c8 = Chip8::new("./examples/maze.ch8");

    let num = 254u8;

    println!("{}", (num / 100) % 10);
    println!("{}", (num / 10) % 10);
    println!("{}", num % 10);
}
