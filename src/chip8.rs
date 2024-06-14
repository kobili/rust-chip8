use rand::Rng;

use crate::constants::{
    FONT_SET,
    FONT_SET_START_ADDRESS,
    PIXEL_OFF,
    PIXEL_ON,
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

// setup methods
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
}

// operation methods
impl Chip8 {
    /// Returns a random byte valued in the range `[0, 255]`
    fn rand_byte(&mut self) -> u8 {
        self.rng.gen::<u8>()
    }

    /// `00E0`: Completely clear the display memory
    fn cls(&mut self) {
        for i in 0..32 {
            for j in 0..64 {
                self.display_memory[i][j] = PIXEL_OFF;
            }
        }
    }

    /// `00EE`: Return from a subroutine
    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    /// `1nnn`: Jump to address `nnn` (`self.pc` -> `nnn`)
    fn jmp(&mut self, opcode: u16) {
        self.pc = opcode & 0x0FFF;
    }

    /// `2nnn`: Call the subroutine at `nnn`
    fn call(&mut self, opcode: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;

        self.pc = opcode & 0x0FFF;
    }

    /// `3xkk`: Skip the next instruction if `Vx == kk`
    fn se_byte(&mut self, opcode: u16) {

    }

    /// `4xkk`: Skip the next instruction if `Vx != kk`
    fn sne_byte(&mut self, opcode: u16) {

    }

    /// `5xy0`: Skip the next instruction if `Vx == Vy`
    fn se_register(&mut self, opcode: u16) {

    }

    /// `6xkk`: Load `kk` into `Vx`
    fn ld_byte(&mut self, opcode: u16) {

    }
    
    /// `7xkk`: Add `kk` with the value stored in `Vx` and store the result in `Vx`
    fn add_byte(&mut self, opcode: u16) {

    }

    /// `8xy0`: Store the value in `Vy` into `Vx`
    fn ld_register(&mut self, opcode: u16) {

    }

    /// `8xy1`: Perform a bitwise OR on the values stored in `Vx`` and `Vy`
    /// then store the result in `Vx`
    fn or(&mut self, opcode: u16) {

    }

    /// `8xy2`: Perform a bitwise AND on the values stored in `Vx` and `Vy`
    /// then store the result in `Vx`
    fn and(&mut self, opcode: u16) {

    }

    /// `8xy3`: Perform a bitwise XOR on the values stored in `Vx`` and `Vy`
    /// then store the result in `Vx`
    fn xor(&mut self, opcode: u16) {

    }

    /// `8xy4`: Perform an addition with the values in `Vx` and `Vy` then store the
    /// result in `Vx`.
    /// 
    /// If the result exceeds the capacity of a u8, `VF` is set to 1, otherwise it is set to 0.
    /// Only the rightmost 8 bits of the result is stored in `Vx`.
    fn add_registers(&mut self, opcode: u16) {

    }

    /// `8xy5`: Subtract `Vx - Vy` and store the result in `Vx`. If `Vx > Vy`, `VF` is set to 1, otherwise 0.
    fn sub_registers(&mut self, opcode: u16) {

    }

    /// `8xy6`: If the least-significant bit of Vy is 1, then VF is set to 1, otherwise 0.
    /// Then Vy is shifted right by 1 and the result is stored in Vx.
    fn shr(&mut self, opcode: u16) {

    }

    /// `8xy7`: Subtract `Vy - Vx` and store the result in `Vx`. If `Vy > Vx`, `VF` is set to 1, otherwise 0.
    fn subn_registers(&mut self, opcode: u16) {

    }

    /// `8xyE`: If the most-significant bit of Vy is 1, then VF is set to 1, otherwise to 0.
    /// Then Vy is shifted left by 1 and the result is stored in Vx.
    fn shl(&mut self, opcode: u16) {

    }

    /// `9xy0`: Skip the next instruction if `Vx != Vy`
    fn sne_register(&mut self, opcode: u16) {

    }

    /// `Annn`: Stores address `nnn` in `self.index_register`
    fn ld_i(&mut self, opcode: u16) {

    }

    /// `Bnnn`: Jump to the address `nnn + V0`
    fn jmp_v0(&mut self, opcode: u16) {

    }

    /// `Cxkk`: Perform a bitwise AND between a random byte and `kk`. Store the value in `Vx`
    /// `Vx -> RAND & kk`
    fn rnd(&mut self, opcode: u16) {

    }

    /// `Dxyn`: Read `n` bytes from memory starting at the address stored in `index_register`.
    /// These bytes are then displayed as sprites on screen at coordinates (`Vx`, `Vy`).
    /// 
    /// Sprites are XOR'd onto the existing screen. If this causes any pixels to be erased, `VF` is
    /// set to 1, otherwise it is set to 0.
    /// 
    /// If the sprite is positioned so part of it is outside the coordinates of the display, it wraps
    /// around to the opposite side of the screen
    fn draw(&mut self, opcode: u16) {

    }

    /// `Ex9E`: Skip the next instruction if the key with value `Vx` is pressed.
    fn skip_key_pressed(&mut self, opcode: u16) {

    }

    /// `ExA1`: Skip the next instruction if the key with value `Vx` is not pressed.
    fn skip_key_not_pressed(&mut self, opcode: u16) {

    }

    /// `Fx07`: Set `Vx -> delay_timer`
    fn ld_delay_timer(&mut self, opcode: u16) {

    }

    /// `Fx0A`: Wait for a key press and store the value of the key in `Vx`
    /// 
    /// All executions stop until a key is pressed.
    fn ld_key_press(&mut self, opcode: u16) {

    }

    /// `Fx15`: Set the delay_timer to the value of `Vx`
    fn set_delay_timer(&mut self, opcode: u16) {

    }

    /// `Fx18`: Set the sound_timer to the value of `Vx`
    fn set_sound_timer(&mut self, opcode: u16) {

    }

    /// `Fx1E`: Add `index_register` and `Vx` and store the result in `index_register`
    fn add_index_register(&mut self, opcode: u16) {

    }

    /// `Fx29`: Load the address of the spirte corresponding to the value of `Vx` into `index_register`.
    fn ld_sprite(&mut self, opcode: u16) {

    }

    /// `Fx33`:  Store BCD (binary-coded decimal) representation of `Vx` in
    /// memory locations `index_register`, `index_register+1`, and `index_register+2`.
    /// 
    /// Take the decimal value of Vx, and place the hundreds digit in memory at location in `index_register`,
    /// the tens digit at location `index_register+1`, and the ones digit at location `index_register+2`.
    fn ld_bcd(&mut self, opcode: u16) {

    }

    /// `Fx55`: Store registers `V0` through `Vx` into memory starting at the address in `index_register`
    fn ld_registers_into_index_register(&mut self, opcode: u16) {

    }

    /// `Fx65`: Read values in memory starting at the address in `index_register`, storing them into registers
    /// `V0` to `Vx`
    fn read_index_register_into_registers(&mut self, opcode: u16) {

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

    #[test]
    fn test_cls() {
        let mut c8 = Chip8::_new();

        // setup: Fill certain pixels on the display
        c8.display_memory[23][35] = PIXEL_ON;
        c8.display_memory[12][63] = PIXEL_ON;
        c8.display_memory[8][40] = PIXEL_ON;

        c8.cls();

        assert_eq!(c8.display_memory[23][35], PIXEL_OFF);
        assert_eq!(c8.display_memory[12][63], PIXEL_OFF);
        assert_eq!(c8.display_memory[8][40], PIXEL_OFF);
    }

    #[test]
    fn test_ret() {
        let mut c8 = Chip8::_new();

        // setup: Point pc to somewhere random and add a return address to the stack
        c8.pc = 0x512;
        c8.stack[0] = 0x208;
        c8.sp = 1;

        c8.ret();

        assert_eq!(c8.pc, 0x208);
        assert_eq!(c8.sp, 0);
    }

    #[test]
    fn test_jmp() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;

        c8.jmp(0x1bea);

        assert_eq!(c8.pc, 0xbea);
    }

    #[test]
    fn test_call() {
        let mut c8 = Chip8::_new();
        c8.pc = 0x220;

        c8.stack[0] = 0x208;
        c8.sp = 1;

        c8.call(0x2512);

        assert_eq!(c8.pc, 0x512);
        assert_eq!(c8.stack[0], 0x208);
        assert_eq!(c8.stack[1], 0x220);
        assert_eq!(c8.sp, 2)
    }

    #[test]
    fn test_se_byte_should_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[10] = 0x32;

        c8.se_byte(0x3a32);

        assert_eq!(c8.pc, 0x0222);
    }

    #[test]
    fn test_se_byte_shouldnt_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[10] = 0x32;

        c8.se_byte(0x3abc);

        assert_eq!(c8.pc, 0x0220);
    }

    #[test]
    fn test_sne_should_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[10] = 0x32;

        c8.se_byte(0x4abc);

        assert_eq!(c8.pc, 0x0222);
    }

    #[test]
    fn test_sne_shouldnt_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[10] = 0x32;

        c8.se_byte(0x4a32);

        assert_eq!(c8.pc, 0x0220);
    }

    #[test]
    fn test_se_register_should_skip() {
        let mut c8 = Chip8::_new();
        c8.pc = 0x220;

        c8.registers[10] = 0x32;
        c8.registers[3] = 0x32;

        c8.se_register(0x5a30);

        assert_eq!(c8.pc, 0x0222);
    }

    #[test]
    fn test_se_register_shouldnt_skip() {
        let mut c8 = Chip8::_new();
        c8.pc = 0x220;

        c8.registers[10] = 0x32;
        c8.registers[3] = 0x31;

        c8.se_register(0x5a30);

        assert_eq!(c8.pc, 0x0220);
    }

    #[test]
    fn test_ld_byte() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;

        c8.ld_byte(0x6abd);

        assert_eq!(c8.registers[0xa], 0xbd);
    }

    #[test]
    fn test_add_byte() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;

        c8.add_byte(0x7a05);

        assert_eq!(c8.registers[0xa], 0x28);
    }

    #[test]
    fn test_ld_register() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;

        c8.ld_register(0x8ad0);

        assert_eq!(c8.registers[0xa], 0x48);
    }

    #[test]
    fn test_or() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;

        c8.or(0x8ad1);

        assert_eq!(c8.registers[0xa], 0x23 | 0x48);
    }

    #[test]
    fn test_and() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xF0;
        c8.registers[0xd] = 0x0F;

        c8.and(0x8ad2);

        assert_eq!(c8.registers[0xa], 0x0);
    }

    #[test]
    fn test_xor() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;

        c8.xor(0x8ad3);

        assert_eq!(c8.registers[0xa], 0x23 ^ 0x48);
    }

    #[test]
    fn test_add_registers() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;
        c8.registers[0xf] = 0x01;

        c8.add_registers(0x8ad4);

        assert_eq!(c8.registers[0xa], 0x23 + 0x48);
        assert_eq!(c8.registers[0xf], 0x0);
    }

    #[test]
    fn test_add_registers_overflow() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xFF;
        c8.registers[0xd] = 0x1;
        c8.registers[0xf] = 0x0;

        c8.add_registers(0x8ad4);

        assert_eq!(c8.registers[0xa], 0x0);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_sub_registers() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xFF;
        c8.registers[0xd] = 0x1;
        c8.registers[0xf] = 0x0;

        c8.sub_registers(0x8ad5);
        
        assert_eq!(c8.registers[0xa], 0xFE);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_sub_registers_underflow() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x2;
        c8.registers[0xd] = 0xFF;
        c8.registers[0xf] = 0x0;

        c8.sub_registers(0x8ad5);
        
        assert_eq!(c8.registers[0xa], 0x1);
        assert_eq!(c8.registers[0xf], 0x0);
    }
}
