#[cfg(test)]
mod test_chip8 {
    use crate::chip8::*;

    #[test]
    fn test_chip8_constructor() {
        let c8 = Chip8::new();
        assert_eq!(c8.pc(), PROGRAM_START_ADDRESS)
    }
}
