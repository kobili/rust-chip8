mod chip8;

fn get_chip8_opcodes_from_file(file_path: &str) -> Vec<u16> {
    let contents = std::fs::read_to_string(file_path).unwrap();

    let mut opcodes = Vec::new();

    for value in contents.trim().replace("\n", " ").split(" ") {
        let hex_value = match u16::from_str_radix(value, 16) {
            Ok(v) => v,
            Err(e) => {
                panic!("Tried converting {} and got {}", value, e)
            }
        };
        opcodes.push(hex_value);
    }

    opcodes
}

fn main() {
    let opcodes = get_chip8_opcodes_from_file("./examples/maze.ch8");

    for code in opcodes {
        println!("{:x}", code);
    }
}
