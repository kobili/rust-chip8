use rand::Rng;
use super::utils::get_bits_of_byte;

use super::constants::{
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
            display_memory: [[PIXEL_OFF; 64]; 32],

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
            display_memory: [[PIXEL_OFF; 64]; 32],

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
    fn cls(&mut self, _opcode: u16) {
        for i in 0..32 {
            for j in 0..64 {
                self.display_memory[i][j] = PIXEL_OFF;
            }
        }
    }

    /// `00EE`: Return from a subroutine
    fn ret(&mut self, _opcode: u16) {
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
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;

        let vx = self.registers[usize::from(x)];
        
        if vx == kk {
            self.pc += 0x02;
        }
    }

    /// `4xkk`: Skip the next instruction if `Vx != kk`
    fn sne_byte(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;

        let vx = self.registers[usize::from(x)];
        if vx != kk {
            self.pc += 0x02;
        }
    }

    /// `5xy0`: Skip the next instruction if `Vx == Vy`
    fn se_register(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        let vx = self.registers[x];
        let vy = self.registers[y];

        if vx == vy {
            self.pc += 2;
        }
    }

    /// `6xkk`: Load `kk` into `Vx`
    fn ld_byte(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let kk = (opcode & 0x00FF) as u8;

        self.registers[x] = kk;
    }
    
    /// `7xkk`: Add `kk` with the value stored in `Vx` and store the result in `Vx`
    fn add_byte(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let kk = (opcode & 0x00FF) as u8;

        self.registers[x] = self.registers[x].wrapping_add(kk);
    }

    /// `8xy0`: Store the value in `Vy` into `Vx`
    fn ld_register(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        self.registers[x] = self.registers[y];
    }

    /// `8xy1`: Perform a bitwise OR on the values stored in `Vx`` and `Vy`
    /// then store the result in `Vx`
    fn or(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        self.registers[x] = self.registers[x] | self.registers[y];
    }

    /// `8xy2`: Perform a bitwise AND on the values stored in `Vx` and `Vy`
    /// then store the result in `Vx`
    fn and(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        self.registers[x] = self.registers[x] & self.registers[y]
    }

    /// `8xy3`: Perform a bitwise XOR on the values stored in `Vx`` and `Vy`
    /// then store the result in `Vx`
    fn xor(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        self.registers[x] = self.registers[x] ^ self.registers[y]
    }

    /// `8xy4`: Perform an addition with the values in `Vx` and `Vy` then store the
    /// result in `Vx`.
    /// 
    /// If the result exceeds the capacity of a u8, `VF` is set to 1, otherwise it is set to 0.
    /// Only the rightmost 8 bits of the result is stored in `Vx`.
    fn add_registers(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        let res = self.registers[x].wrapping_add(self.registers[y]);

        self.registers[0xF] = if res < self.registers[x] { 1 } else { 0 };

        self.registers[x] = res;
    }

    /// `8xy5`: Subtract `Vx - Vy` and store the result in `Vx`. If `Vx > Vy`, `VF` is set to 1, otherwise 0.
    fn sub_registers(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        let vx = self.registers[x];
        let vy = self.registers[y];

        self.registers[0xF] = if vx > vy { 0x1 } else { 0x0 };

        self.registers[x] = vx.wrapping_sub(vy);
    }

    /// `8xy6`: If the least-significant bit of Vy is 1, then VF is set to 1, otherwise 0.
    /// Then Vy is shifted right by 1 and the result is stored in Vx.
    fn shr(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        let vy = self.registers[y];

        let lsb = vy & 0x01;

        self.registers[0xF] = if lsb == 1 { 1 } else { 0 };

        self.registers[x] = vy >> 1;
    }

    /// `8xy7`: Subtract `Vy - Vx` and store the result in `Vx`. If `Vy > Vx`, `VF` is set to 1, otherwise 0.
    fn subn_registers(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        let vx = self.registers[x];
        let vy = self.registers[y];

        self.registers[0xF] = if vy > vx { 0x1 } else { 0x0 };

        self.registers[x] = vy.wrapping_sub(vx);
    }

    /// `8xyE`: If the most-significant bit of Vy is 1, then VF is set to 1, otherwise to 0.
    /// Then Vy is shifted left by 1 and the result is stored in Vx.
    fn shl(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let y = usize::from((opcode & 0x00F0) >> 4);

        let vy = self.registers[y];

        let msb = (vy & 0x80) >> 7;

        self.registers[0xF] = if msb == 1 { 0x1 } else { 0x0 };

        self.registers[x] = vy << 1;
    }

    /// `9xy0`: Skip the next instruction if `Vx != Vy`
    fn sne_register(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;

        let vx = self.registers[usize::from(x)];
        let vy = self.registers[usize::from(y)];

        if vx != vy {
            self.pc += 2;
        }
    }

    /// `Annn`: Stores address `nnn` in `self.index_register`
    fn ld_i(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;

        self.index_register = nnn;
    }

    /// `Bnnn`: Jump to the address `nnn + V0`
    fn jmp_v0(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.pc = nnn + u16::from(self.registers[0x0]);
    }

    /// `Cxkk`: Perform a bitwise AND between a random byte and `kk`. Store the value in `Vx`
    /// `Vx -> RAND & kk`
    fn rand(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        let kk = (opcode & 0x00FF) as u8;

        self.registers[x] = self.rand_byte() & kk;
    }

    /// `Dxyn`: Read `n` bytes from memory starting at the address stored in `index_register`.
    /// These bytes are then displayed as sprites on screen at coordinates (`Vx`, `Vy`).
    /// 
    /// Sprites are represented by a collection of bytes. Each byte represents a row of the sprite, and
    /// each bit in a byte represents a pixel on the screen. For example: a 4 byte sprite will have pixel
    /// dimensions 8x4: 8px wide, 4px tall.
    /// 
    /// Sprites are XOR'd onto the existing screen. If this causes any pixels to be erased, `VF` is
    /// set to 1, otherwise it is set to 0.
    /// 
    /// If the sprite is positioned so part of it is outside the coordinates of the display, it wraps
    /// around to the opposite side of the screen
    /// 
    /// This instruction does not change `index_register`.
    fn draw(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];
        let vy = self.registers[usize::from((opcode & 0x00F0) >> 4)];
        let n = usize::from(opcode & 0x000F);

        // read n bytes from memory starting at index_register
        let mut sprite = vec![0; n];
        for i in 0..n {
            sprite[i] = self.memory[self.index_register as usize + i];
        }

        let mut vf = 0x0_u8;

        // draw the pixels
        for i in 0..n {
            let y = (vy as usize + i) % 32;
            let pixels = get_bits_of_byte(sprite[i]);

            for j in 0..8 {
                let x = (vx as usize + j) % 64;
                let pixel_state = pixels[j];

                if pixel_state == 1_u8 {
                    if self.display_memory[y][x] ^ PIXEL_ON == 0 {
                        // pixel overlaps with another
                        vf = 0x1;
                        self.display_memory[y][x] = PIXEL_OFF;
                    } else {
                        self.display_memory[y][x] = PIXEL_ON;
                    }
                }
            }
        }

        self.registers[0xF] = vf;
    }

    /// `Ex9E`: Skip the next instruction if the key with value `Vx` is pressed.
    fn skip_key_pressed(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];

        if self.keypad[vx as usize] == 1 {
            self.pc += 2;
        }
    }

    /// `ExA1`: Skip the next instruction if the key with value `Vx` is not pressed.
    fn skip_key_not_pressed(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];

        if self.keypad[vx as usize] == 0 {
            self.pc += 2;
        }
    }

    /// `Fx07`: Set `Vx = delay_timer`
    fn ld_delay_timer(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);
        self.registers[x] = self.delay_timer;
    }

    /// `Fx0A`: Wait for a key press and store the value of the key in `Vx`
    /// 
    /// All executions stop until a key is pressed.
    fn ld_key_press(&mut self, opcode: u16) {
        let x = usize::from((opcode & 0x0F00) >> 8);

        for i in 0..self.keypad.len() {
            if self.keypad[i] != 0 {
                self.registers[x] = i as u8;
                return;
            }
        }

        // none of the keys are pressed. Wait by running this op again on the next cycle
        self.pc -= 2;
    }

    /// `Fx15`: Set the delay_timer to the value of `Vx`
    fn set_delay_timer(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];

        self.delay_timer = vx;
    }

    /// `Fx18`: Set the sound_timer to the value of `Vx`
    fn set_sound_timer(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];

        self.sound_timer = vx;
    }

    /// `Fx1E`: Add `index_register` and `Vx` and store the result in `index_register`
    fn add_index_register(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];
        self.index_register = self.index_register.wrapping_add(vx as u16);
    }

    /// `Fx29`: Load the address of the sprite corresponding to the value of `Vx` into `index_register`.
    fn ld_sprite(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];

        self.index_register = u16::from(0x50 + (vx * 5));
    }

    /// `Fx33`:  Store BCD (binary-coded decimal) representation of `Vx` in
    /// memory locations `index_register`, `index_register+1`, and `index_register+2`.
    /// 
    /// Take the decimal value of Vx, and place the hundreds digit in memory at location in `index_register`,
    /// the tens digit at location `index_register+1`, and the ones digit at location `index_register+2`.
    fn ld_bcd(&mut self, opcode: u16) {
        let vx = self.registers[usize::from((opcode & 0x0F00) >> 8)];

        self.memory[usize::from(self.index_register)] = vx / 100;
        self.memory[usize::from(self.index_register + 1)] = (vx / 10) % 10;
        self.memory[usize::from(self.index_register + 2)] = vx % 10;
    }

    /// `Fx55`: Store registers `V0` through `Vx` into memory starting at the address in `index_register`
    fn ld_registers_into_index_register(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;

        for i in 0..=x {
            self.memory[usize::from(self.index_register + i)] = self.registers[i as usize];
        }

        self.index_register = self.index_register + x + 1;
    }

    /// `Fx65`: Read values in memory starting at the address in `index_register`, storing them into registers
    /// `V0` to `Vx`
    fn read_index_register_into_registers(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;

        for i in 0..=x {
            self.registers[i as usize] = self.memory[usize::from(self.index_register + i)];
        }

        self.index_register = self.index_register + x + 1;
    }
}

// opcode decoding and instruction router
impl Chip8 {
    pub fn execute_opcode(&mut self, opcode: u16) {
        // println!("Executing opcode: 0x{:x}", opcode);
        let instruction = match self.select_instruction(opcode) {
            Ok(func) => func,
            Err(e) => panic!("{}", e),
        };

        instruction(self, opcode);
    }

    fn select_instruction(&self, opcode: u16) -> Result<fn(&mut Chip8, u16), &'static str> {
        let first_digit = (opcode & 0xF000) >> 12;
        match first_digit {
            0x0 => self.select_00e_instruction(opcode),
            0x1 => Ok(Chip8::jmp),
            0x2 => Ok(Chip8::call),
            0x3 => Ok(Chip8::se_byte),
            0x4 => Ok(Chip8::sne_byte),
            0x5 => Ok(Chip8::se_register),
            0x6 => Ok(Chip8::ld_byte),
            0x7 => Ok(Chip8::add_byte),
            0x8 => self.select_8_instruction(opcode),
            0x9 => Ok(Chip8::sne_register),
            0xA => Ok(Chip8::ld_i),
            0xB => Ok(Chip8::jmp_v0),
            0xC => Ok(Chip8::rand),
            0xD => Ok(Chip8::draw),
            0xE => self.select_e_instruction(opcode),
            0xF => self.select_f_instruction(opcode),
            _ => Err("No instruction for opcode"),
        }
    }

    fn select_8_instruction(&self, opcode: u16) -> Result<fn(&mut Chip8, u16), &'static str> {
        let last_digit = opcode & 0xF;
        match last_digit {
            0x0 => Ok(Chip8::ld_register),
            0x1 => Ok(Chip8::or),
            0x2 => Ok(Chip8::and),
            0x3 => Ok(Chip8::xor),
            0x4 => Ok(Chip8::add_registers),
            0x5 => Ok(Chip8::sub_registers),
            0x6 => Ok(Chip8::shr),
            0x7 => Ok(Chip8::subn_registers),
            0xE => Ok(Chip8::shl),
            _ => Err("No instruction for opcode"),
        }
    }

    fn select_e_instruction(&self, opcode: u16) -> Result<fn(&mut Chip8, u16), &'static str> {
        let last_two_digits = opcode & 0x00FF;
        match last_two_digits {
            0x9E => Ok(Chip8::skip_key_pressed),
            0xA1 => Ok(Chip8::skip_key_not_pressed),
            _ => Err("No instruction for opcode"),
        }
    }

    fn select_f_instruction(&self, opcode: u16) -> Result<fn(&mut Chip8, u16), &'static str> {
        let last_two_digits = opcode & 0x00FF;

        match last_two_digits {
            0x07 => Ok(Chip8::ld_delay_timer),
            0x0A => Ok(Chip8::ld_key_press),
            0x15 => Ok(Chip8::set_delay_timer),
            0x18 => Ok(Chip8::set_sound_timer),
            0x1E => Ok(Chip8::add_index_register),
            0x29 => Ok(Chip8::ld_sprite),
            0x33 => Ok(Chip8::ld_bcd),
            0x55 => Ok(Chip8::ld_registers_into_index_register),
            0x65 => Ok(Chip8::read_index_register_into_registers),
            _ => Err("No instruction for opcode"),
        }
    }

    fn select_00e_instruction(&self, opcode: u16) -> Result<fn(&mut Chip8, u16), &'static str> {
        let last_digit = opcode & 0xF;

        match last_digit {
            0x0 => Ok(Chip8::cls),
            0xE => Ok(Chip8::ret),
            _ => Err("No instruction for opcode"),
        }
    }
}

// CPU functionality
impl Chip8 {
    pub fn cycle(&mut self) {
        // fetch the next instruction
        let opcode_first_byte = self.memory[self.pc as usize] as u16;
        let opcode_second_byte = self.memory[usize::from(self.pc + 1)] as u16;
        let opcode = (opcode_second_byte << 8) | opcode_first_byte;

        // increment pc before executing
        self.pc += 2;

        self.execute_opcode(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

// accessors
impl Chip8 {
    pub fn get_display_memory(&self) -> [[u32; 64]; 32] {
        self.display_memory
    }

    pub fn get_keypad(&mut self) -> &mut [u8; 16] {
        &mut self.keypad
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    fn print_display_memory(c8: &Chip8) {
        for i in 0..32 {
            let mut row = [0u8; 64];
            for j in 0..64 {
                row[j] = if c8.display_memory[i][j] != 0 { 1 } else { 0 };
            }
            println!("{:?}", row);
        }
    }

    const ZERO_SPRITE: [[u8; 8]; 5] = [
        [1,1,1,1,0,0,0,0],  // 0xF0
        [1,0,0,1,0,0,0,0],  // 0x90
        [1,0,0,1,0,0,0,0],  // 0x90
        [1,0,0,1,0,0,0,0],  // 0x90
        [1,1,1,1,0,0,0,0],  // 0xF0
    ];

    const ONE_SPRITE: [[u8; 8]; 5] = [
        [0,0,1,0,0,0,0,0],  // 0x20
        [0,1,1,0,0,0,0,0],  // 0x60
        [0,0,1,0,0,0,0,0],  // 0x20
        [0,0,1,0,0,0,0,0],  // 0x20
        [0,1,1,1,0,0,0,0],  // 0x70
    ];

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

        c8.execute_opcode(0x00e0);

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

        c8.execute_opcode(0x00ee);

        assert_eq!(c8.pc, 0x208);
        assert_eq!(c8.sp, 0);
    }

    #[test]
    fn test_jmp() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;

        c8.execute_opcode(0x1bea);

        assert_eq!(c8.pc, 0xbea);
    }

    #[test]
    fn test_call() {
        let mut c8 = Chip8::_new();
        c8.pc = 0x220;

        c8.stack[0] = 0x208;
        c8.sp = 1;

        c8.execute_opcode(0x2512);

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

        c8.execute_opcode(0x3a32);

        assert_eq!(c8.pc, 0x0222);
    }

    #[test]
    fn test_se_byte_shouldnt_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[0xa] = 0x32;

        c8.execute_opcode(0x3abc);

        assert_eq!(c8.pc, 0x0220);
    }

    #[test]
    fn test_sne_should_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[0xa] = 0x32;

        c8.execute_opcode(0x4abc);

        assert_eq!(c8.pc, 0x0222);
    }

    #[test]
    fn test_sne_shouldnt_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x220;
        c8.registers[0xa] = 0x32;

        c8.execute_opcode(0x4a32);

        assert_eq!(c8.pc, 0x0220);
    }

    #[test]
    fn test_se_register_should_skip() {
        let mut c8 = Chip8::_new();
        c8.pc = 0x220;

        c8.registers[10] = 0x32;
        c8.registers[3] = 0x32;

        c8.execute_opcode(0x5a30);

        assert_eq!(c8.pc, 0x0222);
    }

    #[test]
    fn test_se_register_shouldnt_skip() {
        let mut c8 = Chip8::_new();
        c8.pc = 0x220;

        c8.registers[10] = 0x32;
        c8.registers[3] = 0x31;

        c8.execute_opcode(0x5a30);

        assert_eq!(c8.pc, 0x0220);
    }

    #[test]
    fn test_ld_byte() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;

        c8.execute_opcode(0x6abd);

        assert_eq!(c8.registers[0xa], 0xbd);
    }

    #[test]
    fn test_add_byte() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;

        c8.execute_opcode(0x7a05);

        assert_eq!(c8.registers[0xa], 0x28);
    }

    #[test]
    fn test_add_byte_overflow() {
        let mut c8 = Chip8::_new();
        c8.registers[0xa] = 0xFF;

        c8.execute_opcode(0x7a01);

        assert_eq!(c8.registers[0xa], 0x0);
    }

    #[test]
    fn test_ld_register() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;

        c8.execute_opcode(0x8ad0);

        assert_eq!(c8.registers[0xa], 0x48);
    }

    #[test]
    fn test_or() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;

        c8.execute_opcode(0x8ad1);

        assert_eq!(c8.registers[0xa], 0x23 | 0x48);
    }

    #[test]
    fn test_and() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xF0;
        c8.registers[0xd] = 0x0F;

        c8.execute_opcode(0x8ad2);

        assert_eq!(c8.registers[0xa], 0x0);
    }

    #[test]
    fn test_xor() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;

        c8.execute_opcode(0x8ad3);

        assert_eq!(c8.registers[0xa], 0x23 ^ 0x48);
    }

    #[test]
    fn test_add_registers() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x23;
        c8.registers[0xd] = 0x48;
        c8.registers[0xf] = 0x01;

        c8.execute_opcode(0x8ad4);

        assert_eq!(c8.registers[0xa], 0x23 + 0x48);
        assert_eq!(c8.registers[0xf], 0x0);
    }

    #[test]
    fn test_add_registers_overflow() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xFF;
        c8.registers[0xd] = 0x1;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0x8ad4);

        assert_eq!(c8.registers[0xa], 0x0);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_sub_registers() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xFF;
        c8.registers[0xd] = 0x1;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0x8ad5);
        
        assert_eq!(c8.registers[0xa], 0xFE);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_sub_registers_underflow() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x2;
        c8.registers[0xd] = 0xFF;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0x8ad5);
        
        assert_eq!(c8.registers[0xa], 0x3);
        assert_eq!(c8.registers[0xf], 0x0);
    }

    #[test]
    fn test_shr_set_vf() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x0;
        c8.registers[0xd] = 0x1f;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0x8ad6);

        assert_eq!(c8.registers[0xa], 0xf);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_shr_dont_set_vf() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x0;
        c8.registers[0xd] = 0x10;
        c8.registers[0xf] = 0x1;

        c8.execute_opcode(0x8ad6);

        assert_eq!(c8.registers[0xa], 0x08);
        assert_eq!(c8.registers[0xf], 0x0);
    }

    #[test]
    fn test_subn_registers() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x02;
        c8.registers[0xb] = 0x0a;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0x8ab7);

        assert_eq!(c8.registers[0xa], 0x08);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_subn_registers_underflow() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x0a;
        c8.registers[0xb] = 0x02;
        c8.registers[0xf] = 0x1;

        c8.execute_opcode(0x8ab7);

        assert_eq!(c8.registers[0xa], 0xf8);
        assert_eq!(c8.registers[0xf], 0x0);
    }

    #[test]
    fn test_shl_set_vf() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xff;
        c8.registers[0xc] = 0x0;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0x8cae);

        assert_eq!(c8.registers[0xc], 0xfe);
        assert_eq!(c8.registers[0xf], 0x1);
    }

    #[test]
    fn test_shl_dont_set_vf() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x7f;
        c8.registers[0xc] = 0x0;
        c8.registers[0xf] = 0x1;

        c8.execute_opcode(0x8cae);

        assert_eq!(c8.registers[0xc], 0xfe);
        assert_eq!(c8.registers[0xf], 0x0);
    }

    #[test]
    fn test_sne_register_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x206;
        c8.registers[0xa] = 0x78;
        c8.registers[0xb] = 0x98;

        c8.execute_opcode(0x9ab0);
    
        assert_eq!(c8.pc, 0x208);
    }

    #[test]
    fn test_sne_register_dont_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x206;
        c8.registers[0xa] = 0x78;
        c8.registers[0xb] = 0x78;

        c8.execute_opcode(0x9ab0);
    
        assert_eq!(c8.pc, 0x206);
    }

    #[test]
    fn test_ld_i() {
        let mut c8 = Chip8::_new();

        c8.index_register = 0x512;

        c8.execute_opcode(0xaabc);

        assert_eq!(c8.index_register, 0xabc);
    }

    #[test]
    fn test_jmp_v0() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x224;
        c8.registers[0] = 0x10;

        c8.execute_opcode(0xBabc);

        assert_eq!(c8.pc, 0xacc);
    }

    #[test]
    fn test_draw() {
        let mut c8 = Chip8::_new();
        c8.load_fontset();

        // load a 1 on screen starting at (0, 0)
        // the 1 sprite is located at 0x55 in memory
        c8.index_register = 0x55;
        c8.registers[0xa] = 0x0;
        c8.registers[0xb] = 0x0;
        c8.registers[0xf] = 0x1;
        
        c8.execute_opcode(0xdab5);

        for i in 0..5 {
            for j in 0..8 {
                let expected_value = if ONE_SPRITE[i][j] == 1 { PIXEL_ON } else { PIXEL_OFF };
                assert_eq!(c8.display_memory[i][j], expected_value);
            }
        }
        assert_eq!(c8.registers[0xf], 0x0);
        assert_eq!(c8.index_register, 0x55);
    }

    #[test]
    fn test_draw_overlapping_sprites() {
        let mut c8 = Chip8::_new();
        c8.load_fontset();

        // preload a sprite at (1, 1)
        for i in 0..5 {
            for j in 0..8 {
                c8.display_memory[i+1][j+1] = if ZERO_SPRITE[i][j] == 1u8 { PIXEL_ON } else { PIXEL_OFF };
            }
        }

        c8.index_register = 0x50;
        c8.registers[0xa] = 0x1;
        c8.registers[0xb] = 0x1;
        c8.registers[0xf] = 0x0;

        c8.execute_opcode(0xdab5);

        assert_eq!(c8.registers[0xf], 0x1);

        // our draw operation should have erased our existing 0 sprite
        for i in 0..32 {
            for j in 0..64 {
                assert_eq!(c8.display_memory[i][j], PIXEL_OFF);
            }
        }

        assert_eq!(c8.index_register, 0x50);
    }

    #[test]
    fn test_draw_overflowing_sprites() {
        let mut c8 = Chip8::_new();
        c8.load_fontset();


        let mut expected_display = [[PIXEL_OFF; 64]; 32];

        let x = 61;
        let y = 29;

        for i in 0..5 {
            for j in 0..8 {
                expected_display[(y + i) % 32][(x + j) % 64] = if ZERO_SPRITE[i][j] == 1 { PIXEL_ON } else { PIXEL_OFF };
            }
        }

        c8.index_register = 0x50;
        c8.registers[0xa] = x as u8;
        c8.registers[0xb] = y as u8;
        c8.registers[0xf] = 0x1;

        c8.execute_opcode(0xdab5);

        assert_eq!(c8.index_register, 0x50);
        assert_eq!(c8.registers[0xf], 0x0);

        for i in 0..32 {
            for j in 0..64 {
                assert_eq!(c8.display_memory[i][j], expected_display[i][j]);
            }
        }
    }

    #[test]
    fn test_skip_key_pressed() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x224;
        c8.keypad[2] = 1;
        c8.registers[0xa] = 2;

        c8.execute_opcode(0xea9e);

        assert_eq!(c8.pc, 0x226);
    }

    #[test]
    fn test_skip_key_pressed_no_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x224;
        c8.keypad[2] = 0;
        c8.registers[0xa] = 2;

        c8.execute_opcode(0xea9e);

        assert_eq!(c8.pc, 0x224);
    }

    #[test]
    fn test_skip_key_not_pressed() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x224;
        c8.keypad[2] = 0;
        c8.registers[0xa] = 2;

        c8.execute_opcode(0xeaa1);

        assert_eq!(c8.pc, 0x226);
    }

    #[test]
    fn test_skip_key_not_pressed_no_skip() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x224;
        c8.keypad[2] = 1;
        c8.registers[0xa] = 2;

        c8.execute_opcode(0xeaa1);

        assert_eq!(c8.pc, 0x224);
    }

    #[test]
    fn test_ld_delay_timer() {
        let mut c8 = Chip8::_new();
        c8.delay_timer = 0x20;

        c8.execute_opcode(0xfa07);

        assert_eq!(c8.registers[0xa], 0x20);
    }

    #[test]
    fn test_set_delay_timer() {
        let mut c8 = Chip8::_new();
        c8.registers[0xa] = 0x50;

        c8.execute_opcode(0xfa15);

        assert_eq!(c8.delay_timer, 0x50);
    }

    #[test]
    fn test_ld_key_press() {
        let mut c8 = Chip8::_new();

        c8.keypad[0xf] = 1;

        c8.execute_opcode(0xfa0a);

        assert_eq!(c8.registers[0xa], 0xf);
    }

    #[test]
    fn test_ld_key_press_no_keys_pressed() {
        let mut c8 = Chip8::_new();

        c8.pc = 0x204;
        c8.execute_opcode(0xfa0a);

        assert_eq!(c8.pc, 0x202);
    }

    #[test]
    fn test_set_sound_timer() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x20;

        c8.execute_opcode(0xfa18);

        assert_eq!(c8.sound_timer, 0x20);
    }

    #[test]
    fn test_add_index_register() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x2;
        c8.index_register = 0x220;

        c8.execute_opcode(0xfa1e);

        assert_eq!(c8.index_register, 0x222);
    }

    #[test]
    fn test_add_index_register_overflow() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x2;
        c8.index_register = 0xFFFF;

        c8.execute_opcode(0xfa1e);

        assert_eq!(c8.index_register, 0x1);
    }

    #[test]
    fn test_ld_sprite() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x4;

        c8.execute_opcode(0xfa29);

        assert_eq!(c8.index_register, 0x64);
    }

    #[test]
    fn test_ld_sprite_zero() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x0;

        c8.execute_opcode(0xfa29);

        assert_eq!(c8.index_register, 0x50);
    }

    #[test]
    fn test_ld_bcd() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0xFE; // 254
        c8.index_register = 0x100;

        c8.execute_opcode(0xfa33);

        assert_eq!(c8.memory[0x100], 2);
        assert_eq!(c8.memory[0x101], 5);
        assert_eq!(c8.memory[0x102], 4);
    }

    #[test]
    fn test_ld_bcd_double_digits() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x10; // 16
        c8.index_register = 0x100;

        c8.execute_opcode(0xfa33);

        assert_eq!(c8.memory[0x100], 0);
        assert_eq!(c8.memory[0x101], 1);
        assert_eq!(c8.memory[0x102], 6);
    }

    #[test]
    fn test_ld_bcd_single_digits() {
        let mut c8 = Chip8::_new();

        c8.registers[0xa] = 0x2; // 2
        c8.index_register = 0x100;

        c8.execute_opcode(0xfa33);

        assert_eq!(c8.memory[0x100], 0);
        assert_eq!(c8.memory[0x101], 0);
        assert_eq!(c8.memory[0x102], 2);
    }

    #[test]
    fn test_ld_registers_into_index_register() {
        let mut c8 = Chip8::_new();

        c8.registers[0x0] = 0x1;
        c8.registers[0x1] = 0x2;
        c8.registers[0x2] = 0x3;
        c8.registers[0x3] = 0x4;
        c8.registers[0x4] = 0x5;
        c8.index_register = 0x100;

        c8.execute_opcode(0xf455);

        assert_eq!(c8.index_register, 0x105);

        assert_eq!(c8.memory[0x100], 0x1);
        assert_eq!(c8.memory[0x101], 0x2);
        assert_eq!(c8.memory[0x102], 0x3);
        assert_eq!(c8.memory[0x103], 0x4);
        assert_eq!(c8.memory[0x104], 0x5);
    }

    #[test]
    fn test_read_index_register_into_registers() {
        let mut c8 = Chip8::_new();

        c8.memory[0x100] = 0x1;
        c8.memory[0x101] = 0x2;
        c8.memory[0x102] = 0x3;
        c8.memory[0x103] = 0x4;
        c8.memory[0x104] = 0x5;

        c8.index_register = 0x100;

        c8.execute_opcode(0xf465);

        assert_eq!(c8.index_register, 0x105);

        assert_eq!(c8.registers[0], 0x1);
        assert_eq!(c8.registers[1], 0x2);
        assert_eq!(c8.registers[2], 0x3);
        assert_eq!(c8.registers[3], 0x4);
        assert_eq!(c8.registers[4], 0x5);
    }

    #[test]
    fn test_cycle() {
        let mut c8 = Chip8::_new();

        // load an instruction into memory and set it to the next op
        c8.pc = 0x200;
        c8.memory[0x200] = 0xb5;
        c8.memory[0x201] = 0xda;

        // set delay and sound timers
        c8.delay_timer = 3;
        c8.sound_timer = 4;

        // use setup for test_draw_overlapping_sprites
        c8.load_fontset();
        for i in 0..5 {
            for j in 0..8 {
                c8.display_memory[i+1][j+1] = if ZERO_SPRITE[i][j] == 1u8 { PIXEL_ON } else { PIXEL_OFF };
            }
        }
        c8.index_register = 0x50;
        c8.registers[0xa] = 0x1;
        c8.registers[0xb] = 0x1;
        c8.registers[0xf] = 0x0;

        c8.cycle();

        // assert cycle operations
        assert_eq!(c8.pc, 0x202);
        assert_eq!(c8.delay_timer, 2);
        assert_eq!(c8.sound_timer, 3);

        // assertions from test_draw_overlapping_sprites
        assert_eq!(c8.registers[0xf], 0x1);
        for i in 0..32 {
            for j in 0..64 {
                assert_eq!(c8.display_memory[i][j], PIXEL_OFF);
            }
        }
        assert_eq!(c8.index_register, 0x50);
    }
}
