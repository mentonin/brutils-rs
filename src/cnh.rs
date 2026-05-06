use crate::common;
use std::fmt::Display;
use std::str::FromStr;

/// A structure that holds only numerically valid CNH numbers.
/// No effort is made to verify that the CNH actually exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cnh([u8; Self::SIZE]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCnhError {
    WrongLength,
    NonNumeric,
    WrongChecksum,
}

impl Cnh {
    const SIZE: usize = 11;

    /// Generates a random, numerically valid CNH.
    /// The generated CNH may or may not exist.
    pub fn generate() -> Self {
        use rand::distributions::{Distribution, Uniform};

        let mut digit_dist = Uniform::from(0..=9u8).sample_iter(rand::thread_rng());
        let mut num = [0u8; 11];

        while num[0..9].iter().all(|&x| x == num[0]) {
            num[0..9].copy_from_slice(&digit_dist.by_ref().take(9).collect::<Vec<u8>>());
        }

        let checksum = Self::compute_checksum(num.first_chunk().unwrap());
        num[9..].copy_from_slice(&checksum);
        Self(num)
    }

    /// Computes the 2-digit CNH checksum for a 9-digit base number.
    fn compute_checksum(base: &[u8; 9]) -> [u8; 2] {
        use common::modulo11_gen;
        let d1 = modulo11_gen(base) % 10;
        let d2 = modulo11_gen(base.iter().chain(&[d1])) % 10;
        [d1, d2]
    }
}

impl FromStr for Cnh {
    type Err = ParseCnhError;

    /// Tries to parse a string into a numerically valid Cnh number.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != Self::SIZE {
            return Err(ParseCnhError::WrongLength);
        }

        let mut digits = [0; Self::SIZE];
        for (c, d) in s.chars().zip(digits.iter_mut()) {
            *d = c.to_digit(10).ok_or(ParseCnhError::NonNumeric)? as u8;
        }

        if &Self::compute_checksum(digits.first_chunk().unwrap()) != digits.last_chunk().unwrap() {
            return Err(ParseCnhError::WrongChecksum);
        }

        Ok(Self(digits))
    }
}

impl Display for Cnh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let digits: String = self.0.iter().map(|d| (d + b'0') as char).collect();
        write!(f, "{digits}",)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_cnh() {
        // Invalid: repeated sequence
        assert!(Cnh::from_str("22222222222").is_err());
        assert!(Cnh::from_str("00000000000").is_err());
        assert!(Cnh::from_str("11111111111").is_err());
        assert!(Cnh::from_str("33333333333").is_err());
        assert!(Cnh::from_str("99999999999").is_err());

        // Invalid: contains letters
        assert!(Cnh::from_str("ABC70304734").is_err());
        assert!(Cnh::from_str("A2C45678901").is_err());
        assert!(Cnh::from_str("1234567890A").is_err());

        // Invalid: wrong length
        assert!(Cnh::from_str("6619558737912").is_err());
        assert!(Cnh::from_str("123456789").is_err());
        assert!(Cnh::from_str("1234567890").is_err());
        assert!(Cnh::from_str("123456789012").is_err());

        // Valid with formatting
        assert!(Cnh::from_str("097703047-34").is_ok());
        assert!(Cnh::from_str("987654321-00").is_ok());

        // Valid without formatting
        assert!(Cnh::from_str("09770304734").is_ok());
        assert!(Cnh::from_str("98765432100").is_ok());

        // Additional test cases - invalid checksum
        assert!(Cnh::from_str("12345678901").is_err());

        // Edge cases
        assert!(Cnh::from_str("").is_err());
        assert!(Cnh::from_str("           ").is_err());
        assert!(Cnh::from_str("---").is_err());
    }

    #[test]
    fn test_is_valid_cnh_symbols_removed() {
        // Test that various symbols are removed
        assert!(Cnh::from_str("097-703-047-34").is_ok());
        assert!(Cnh::from_str("097.703.047.34").is_ok());
        assert!(Cnh::from_str("097 703 047 34").is_ok());
        assert!(Cnh::from_str("(097)703-047-34").is_ok());
    }

    #[test]
    fn test_is_valid_cnh_mixed_invalid() {
        // Mixed letters and numbers
        assert!(Cnh::from_str("0977O3O4734").is_err()); // O instead of 0
        assert!(Cnh::from_str("097703O4734").is_err());
    }
}
