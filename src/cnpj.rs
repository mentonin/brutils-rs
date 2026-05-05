use crate::common;
use std::fmt::Display;
use std::str::FromStr;

pub struct Cnpj([u8; Self::SIZE]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCnpjError {
    WrongLength,
    NonAlphanumeric,
    WrongChecksum,
}

impl Cnpj {
    const SIZE: usize = 14;

    pub fn generate(branch: Option<u32>) -> Self {
        use rand::distributions::{Distribution, Uniform};

        let mut rng = rand::thread_rng();
        let digit_dist = Uniform::from(0..=36u8);
        let mut num = [0u8; 14];

        while num[0..8].iter().all(|&x| x == num[0]) {
            num[0..12].copy_from_slice(
                &digit_dist
                    .sample_iter(&mut rng)
                    .take(12)
                    .collect::<Vec<u8>>(),
            );
        }

        if let Some(mut branch_num) = branch {
            branch_num %= 1000;
            for d in num[8..12].iter_mut().rev() {
                *d = (branch_num % 10) as u8;
                branch_num /= 10;
            }
        }

        if num[8..12].iter().all(|&x| x == 0) {
            num[11] = 1
        }

        let checksum = Self::compute_checksum(num.first_chunk().unwrap());
        num[12..].copy_from_slice(&checksum);
        Self(num)
    }

    fn compute_checksum(base: &[u8; 12]) -> [u8; 2] {
        let mut base = base.to_vec();
        fn hash_digit(base: &[u8]) -> u8 {
            let mod_diff = base
                .rchunks(8)
                .fold(0, |acc, b| (acc + common::modulo11_gen(b)) % 11);
            mod_diff % 10
        }
        let d1 = hash_digit(&base);
        base.push(d1);
        let d2 = hash_digit(&base);
        [d1, d2]
    }
}

impl FromStr for Cnpj {
    type Err = ParseCnpjError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = common::remove_symbols(s, ".-/ ");
        if s.len() != Self::SIZE {
            return Err(ParseCnpjError::WrongLength);
        }

        let mut digits = [0; Self::SIZE];
        for (c, d) in s.chars().zip(digits.iter_mut()) {
            *d = c.to_digit(10).ok_or(ParseCnpjError::NonAlphanumeric)? as u8;
        }

        if Cnpj::compute_checksum(digits.first_chunk().unwrap()) != digits[12..] {
            return Err(ParseCnpjError::WrongChecksum);
        }

        Ok(Cnpj(digits))
    }
}

impl Display for Cnpj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let digits: String = self.0.iter().map(|d| (d + b'0') as char).collect();
        write!(
            f,
            "{}.{}.{}/{}-{}",
            &digits[0..2],
            &digits[2..5],
            &digits[5..8],
            &digits[8..12],
            &digits[12..14],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        // Valid CNPJs should be formatted
        assert_eq!(
            Cnpj::from_str("03560714000142").unwrap().to_string(),
            "03.560.714/0001-42"
        );
        assert_eq!(
            Cnpj::from_str("01838723000127").unwrap().to_string(),
            "01.838.723/0001-27"
        );
        assert_eq!(
            Cnpj::from_str("34665388000161").unwrap().to_string(),
            "34.665.388/0001-61"
        );
    }

    #[test]
    fn test_validate() {
        // Valid CNPJs
        assert!(Cnpj::from_str("34665388000161").is_ok());
        assert!(Cnpj::from_str("03560714000142").is_ok());
        assert!(Cnpj::from_str("01838723000127").is_ok());

        // Invalid CNPJs
        assert!(Cnpj::from_str("52599927000100").is_err());
        assert!(Cnpj::from_str("00000000000").is_err());
        assert!(Cnpj::from_str("00000000000000").is_err());
        assert!(Cnpj::from_str("11111111111111").is_err());
        assert!(Cnpj::from_str("00111222000133").is_err());
    }

    #[test]
    fn test_is_valid() {
        // When CNPJ's len is different of 14, returns False
        assert!(Cnpj::from_str("1").is_err());

        // When CNPJ does not contain only digits, returns False
        assert!(Cnpj::from_str("1112223334445-").is_err());

        // When CNPJ has only the same digit, returns false
        assert!(Cnpj::from_str("11111111111111").is_err());

        // When rest_1 is lt 2 and the 13th digit is not 0, returns False
        assert!(Cnpj::from_str("1111111111315").is_err());

        // When rest_1 is gte 2 and the 13th digit is not (11 - rest), returns False
        assert!(Cnpj::from_str("1111111111115").is_err());

        // When rest_2 is lt 2 and the 14th digit is not 0, returns False
        assert!(Cnpj::from_str("11111111121205").is_err());

        // When rest_2 is gte 2 and the 14th digit is not (11 - rest), returns False
        assert!(Cnpj::from_str("11111111113105").is_err());

        // When CNPJ is valid
        assert!(Cnpj::from_str("34665388000161").is_ok());
        assert!(Cnpj::from_str("01838723000127").is_ok());
    }

    #[test]
    fn test_generate() {
        // Test that generate creates valid CNPJs
        for _ in 0..1000 {
            let cnpj = Cnpj::generate(None);
            assert_eq!(
                &Cnpj::compute_checksum(cnpj.0.first_chunk().unwrap()),
                cnpj.0.last_chunk().unwrap()
            );
        }
    }

    #[test]
    fn test_compute_checksum() {
        assert_eq!(
            Cnpj::compute_checksum(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            [0, 0]
        );
        assert_eq!(
            Cnpj::compute_checksum(&[5, 2, 5, 1, 3, 1, 2, 7, 0, 0, 0, 2]),
            [9, 9]
        );
        assert_eq!(
            Cnpj::compute_checksum(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2]),
            [3, 0]
        );
    }

    #[test]
    fn test_edge_cases() {
        // Empty string
        assert!(Cnpj::from_str("").is_err());

        // Too short
        assert!(Cnpj::from_str("123456789012").is_err());

        // Too long
        assert!(Cnpj::from_str("123456789012345").is_err());

        // Contains letters
        assert!(Cnpj::from_str("1234567890123a").is_err());

        // All same digit
        assert!(Cnpj::from_str("00000000000000").is_err());
        assert!(Cnpj::from_str("99999999999999").is_err());
    }

    #[test]
    fn test_generate_with_zero_branch() {
        // Branch 0 should become 1
        let cnpj = Cnpj::generate(Some(0));
        assert_eq!(
            &Cnpj::compute_checksum(cnpj.0.first_chunk().unwrap()),
            cnpj.0.last_chunk().unwrap()
        );
        // Branch should be "0001"
        assert_eq!(&cnpj.0[8..12], [0, 0, 0, 1]);
    }

    #[test]
    fn test_generate_branch_modulo() {
        // Branch larger than 9999 should wrap around
        let cnpj = Cnpj::generate(Some(10000));
        assert_eq!(
            &Cnpj::compute_checksum(cnpj.0.first_chunk().unwrap()),
            cnpj.0.last_chunk().unwrap()
        );
        // Should wrap to 0, then become 1
        assert_eq!(&cnpj.0[8..12], [0, 0, 0, 1]);
    }
}
