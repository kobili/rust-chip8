const PROGRAM_START_ADDRESS: u16 = 0x200;

#[derive(Debug)]
pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index_register: u16,
    pc: u16,            // program counter
    stack: [u16; 16],
    sp: u8,             // stack pointer
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    display_memory: [[u32; 64]; 32],

    opcode: u16,
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            registers: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            pc: PROGRAM_START_ADDRESS,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            display_memory: [[0; 64]; 32],
            opcode: 0,
        }
    }

    pub fn load_instructions_from_file(&mut self, file_path: &str) {
        let contents = std::fs::read_to_string(file_path).unwrap();

        let mut opcodes = Vec::new();

        for value in contents.trim().replace("\n", " ").split(" ") {
            let hex_value = match u16::from_str_radix(value, 16) {
                Ok(v) => v,
                Err(e) => {
                    panic!("Error parsing instruction file. Tried converting {} and got {}", value, e)
                }
            };
            opcodes.push(hex_value);
        }

        self.load_opcodes_into_memory(&opcodes);
    }

    fn load_opcodes_into_memory(&mut self, opcodes: &Vec<u16>) {
        let mut i = PROGRAM_START_ADDRESS as usize;

        for opcode in opcodes {
            let little_end = (*opcode & 0x00FF) as u8;
            let big_end = ((*opcode & 0xFF00) >> 8) as u8;

            self.memory[i] = little_end;
            self.memory[i+1] = big_end;

            i += 2;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip8_constructor() {
        let c8 = Chip8::new();
        assert_eq!(c8.pc, PROGRAM_START_ADDRESS)
    }

    #[test]
    fn test_load_opcodes_into_memory() {
        let mut c8 = Chip8::new();
        let opcodes = vec![0x6000u16, 0x6100u16, 0xa222u16];

        c8.load_opcodes_into_memory(&opcodes);

        let expected_byte_order = vec![
            0x00u8,
            0x60u8,
            0x00u8,
            0x61u8,
            0x22u8,
            0xa2u8,
        ];

        // assert that nothing is loaded into reserved memory
        for i in 0..PROGRAM_START_ADDRESS as usize {
            assert_eq!(c8.memory[i], 0);
        }

        // assert that instructions are loaded into memory starting at 0x2000
        for i in 0..(opcodes.len() * 2) {
            let address = i + PROGRAM_START_ADDRESS as usize;
            assert_eq!(c8.memory[address], expected_byte_order[i]);
        }
    }
}
