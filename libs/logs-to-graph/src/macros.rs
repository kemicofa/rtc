#[macro_export]
macro_rules! hash {
    ($input:expr) => {
        {
            let s: &str = $input;
            let hash = blake3::hash(s.as_bytes());
            hash.to_hex().to_string()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::super::hash;

    #[test]
    fn should_hash() {
        assert_eq!(
            hash!("hello"),
            "ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f"
        );
    }
}
