pub fn get_bits_of_byte(byte: u8) -> [u8; 8] {
    let mut bits: [u8; 8] = [0; 8];
    for i in 0..8 {
        bits[7 - i] = (byte & (1 << i)) >> i;
    }

    bits
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn test_get_bits_of_byte() {
        let mut test_cases: HashMap<u8, Vec<u8>> = HashMap::new();
        test_cases.insert(0xFF, vec![1,1,1,1,1,1,1,1]);
        test_cases.insert(0x9F, vec![1,0,0,1,1,1,1,1]);
        test_cases.insert(0xFA, vec![1,1,1,1,1,0,1,0]);

        for (input, expected) in &test_cases {
            let actual = get_bits_of_byte(*input);

            for i in 0..8 {
                assert_eq!(actual[i], expected[i]);
            }
        }
    }
}
