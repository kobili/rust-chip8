#[cfg(test)]
mod test_math {
    #[test]
    fn test_add() {
        assert_eq!(2, 1+1);
    }

    #[test]
    fn test_bad_add() {
        assert_ne!(2 + 2, 5);
    }
}