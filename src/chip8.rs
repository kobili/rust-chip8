use rand::Rng;

use crate::constants::{
    FONT_SET,
    FONT_SET_START_ADDRESS,
    PROGRAM_START_ADDRESS,
};

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

    rng: rand::rngs::ThreadRng,
}

impl Chip8 {
    fn _new() -> Self {
        Self {
            registers: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            pc: PROGRAM_START_ADDRESS as u16,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            display_memory: [[0; 64]; 32],
            opcode: 0,

            rng: rand::thread_rng(),
        }
    }

    /// Creates and initializes a new Chip8 instance with the given instructions
    pub fn new(instruction_file: &str) -> Self {
        let mut c8 = Self {
            registers: [0; 16],
            memory: [0; 4096],
            index_register: 0,
            pc: PROGRAM_START_ADDRESS as u16,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            display_memory: [[0; 64]; 32],

            opcode: 0,
            rng: rand::thread_rng(),
        };
        c8.load_fontset();
        c8.load_instructions_from_file(instruction_file);

        c8
    }

    /// Reads instructions from a `.ch8` file and loads the opcodes into memory
    fn load_instructions_from_file(&mut self, file_path: &str) {
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

    /// Given an array of 16-bit opcodes, loads them into memory
    /// starting at address 0x200, in little-endian order
    fn load_opcodes_into_memory(&mut self, opcodes: &Vec<u16>) {
        let mut i = PROGRAM_START_ADDRESS;

        for opcode in opcodes {
            let little_end = (*opcode & 0x00FF) as u8;
            let big_end = ((*opcode & 0xFF00) >> 8) as u8;

            self.memory[i] = little_end;
            self.memory[i+1] = big_end;

            i += 2;
        }
    }

    /// Loads fontsets into memory starting at address 0x50
    fn load_fontset(&mut self) {
        for i in 0..FONT_SET.len() {
            self.memory[FONT_SET_START_ADDRESS + i] = FONT_SET[i];
        }
    }

    /// Returns a random byte valued in the range `[0, 255]`
    fn rand_byte(&mut self) -> u8 {
        self.rng.gen::<u8>()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip8_constructor() {
        let c8 = Chip8::_new();
        assert_eq!(c8.pc, 0x200);
    }

    #[test]
    fn test_load_opcodes_into_memory() {
        let mut c8 = Chip8::_new();
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
        for i in 0..PROGRAM_START_ADDRESS {
            assert_eq!(c8.memory[i], 0);
        }

        // assert that instructions are loaded into memory starting at 0x2000
        for i in 0..(opcodes.len() * 2) {
            let address = i + PROGRAM_START_ADDRESS;
            assert_eq!(c8.memory[address], expected_byte_order[i]);
        }
    }

    #[test]
    fn test_load_fontset() {
        let mut c8 = Chip8::_new();

        c8.load_fontset();

        for i in 0..FONT_SET.len() {
            let address = FONT_SET_START_ADDRESS + i;
            assert_eq!(c8.memory[address], FONT_SET[i]);
        }
    }
}
