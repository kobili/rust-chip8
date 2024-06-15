mod chip8;
mod constants;
mod utils;

use chip8::Chip8;


#[allow(arithmetic_overflow)]
fn main() {
    let _c8 = Chip8::new("./examples/maze.ch8");

    let mut overlap = false;

    overlap = overlap || (constants::PIXEL_OFF ^ constants::PIXEL_OFF == 0);
    println!("{}", overlap);
}
