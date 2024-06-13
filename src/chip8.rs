pub const PROGRAM_START_ADDRESS: u16 = 0x200;

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

    pub fn opcode_at_start_address(&self) -> u8 {
        self.memory[PROGRAM_START_ADDRESS as usize]
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }
}
