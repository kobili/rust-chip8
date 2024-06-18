mod emulator;
mod screen;

use emulator::chip8::Chip8;

fn main() {
    let mut c8 = Chip8::new("./examples/maze.ch8");
    c8.execute_opcode(0xfa18);

    let display_mem = c8.get_display_memory();

    let opcode = 0xA222;
    println!("{:x}", (opcode & 0xF000) >> 12);

    screen::draw_screen();
}
