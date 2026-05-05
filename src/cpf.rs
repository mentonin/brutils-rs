use std::fmt::Display;
use std::str::FromStr;

use crate::common;

/// A structure that holds only numerically valid CPF numbers.
/// No effort is made to verify that the CPF actually exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cpf([u8; Self::SIZE]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCpfError {
    WrongLength,
    NonNumeric,
    WrongChecksum,
    BlackListed,
}

impl Cpf {
    const SIZE: usize = 11;

    const BLACKLIST: [&str; 10] = [
        "00000000000",
        "11111111111",
        "22222222222",
        "33333333333",
        "44444444444",
        "55555555555",
        "66666666666",
        "77777777777",
        "88888888888",
        "99999999999",
    ];

    /// Generates a random, numerically valid CPF.
    /// The generated CPF may or may not exist.
    pub fn generate() -> Self {
        use rand::distributions::{Distribution, Uniform};

        let mut rng = rand::thread_rng();
        let digit_dist = Uniform::from(0..=9u8);
        let mut num = [0u8; 11];

        // random base, reroll blacklisted
        while num[0..9].iter().all(|&x| x == num[0]) {
            num[0..9].copy_from_slice(
                &digit_dist
                    .sample_iter(&mut rng)
                    .take(9)
                    .collect::<Vec<u8>>(),
            );
        }

        let checksum = Cpf::compute_checksum(num[0..9].as_array().unwrap());
        num[9..].copy_from_slice(&checksum);
        Self(num)
    }

    /// Computes the 2-digit CPF checksum for a 9-digit base number.
    fn compute_checksum(base: &[u8; 9]) -> [u8; 2] {
        use common::modulo11_gen;
        let d1 = modulo11_gen(base) % 10;
        let d2 = modulo11_gen(base.iter().chain(&[d1])) % 10;
        [d1, d2]
    }
}

impl FromStr for Cpf {
    type Err = ParseCpfError;

    /// Tries to parse a string into a numerically valid CPF number.
    /// Trims whitespace and ignores periods and dashes.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = common::remove_symbols(s, ".- ");
        if s.len() != Self::SIZE {
            return Err(ParseCpfError::WrongLength);
        }

        if Self::BLACKLIST.contains(&s.as_str()) {
            return Err(ParseCpfError::BlackListed);
        }

        let mut digits = [0; Self::SIZE];
        for (c, d) in s.chars().zip(digits.iter_mut()) {
            *d = c.to_digit(10).ok_or(ParseCpfError::NonNumeric)? as u8;
        }

        if Cpf::compute_checksum(digits[..9].as_array().unwrap()) != digits[9..] {
            return Err(ParseCpfError::WrongChecksum);
        }

        Ok(Cpf(digits))
    }
}

impl Display for Cpf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let digits: String = self.0.iter().map(|d| (d + b'0') as char).collect();
        write!(
            f,
            "{}.{}.{}-{}",
            &digits[0..3],
            &digits[3..6],
            &digits[6..9],
            &digits[9..11],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CPF_LIST: [&str; 12] = [
        "11144477735",
        "111.444.777-35",
        "111-444-777-35",
        "111.444.777.35",
        "111444777-35",
        "  111444777-35  ",
        "40364478829",
        "52513127765",
        "52599927765",
        "55550207753",
        "82178537464",
        "96271845860",
    ];

    const WRONG_LENGTH_LIST: [&str; 3] = ["1", "1234567890", "123456789012"];

    const NON_DIGITS_LIST: [&str; 3] = ["b1144477735", "1234567890a", "12345!78901"];

    const WRONG_CHECKSUM_LIST: [&str; 3] = ["11144477705", "11144477732", "11111111215"];

    #[test]
    fn test_parse_valid() {
        for s in VALID_CPF_LIST {
            assert!(s.parse::<Cpf>().is_ok());
        }
    }

    #[test]
    fn test_parse_wrong_length() {
        for s in WRONG_LENGTH_LIST {
            assert_eq!(
                s.parse::<Cpf>(),
                Err(ParseCpfError::WrongLength),
                "failed when parsing '{s}'"
            );
        }
    }

    #[test]
    fn test_parse_non_digits() {
        for s in NON_DIGITS_LIST {
            assert_eq!(
                s.parse::<Cpf>(),
                Err(ParseCpfError::NonNumeric),
                "failed when parsing '{s}'"
            );
        }
    }

    #[test]
    fn test_parse_wrong_checksum() {
        for s in WRONG_CHECKSUM_LIST {
            assert_eq!(
                s.parse::<Cpf>(),
                Err(ParseCpfError::WrongChecksum),
                "failed when parsing '{s}'"
            );
        }
    }

    #[test]
    fn test_parse_blacklisted() {
        for s in Cpf::BLACKLIST {
            assert_eq!(
                s.parse::<Cpf>(),
                Err(ParseCpfError::BlackListed),
                "failed when parsing '{s}'"
            );
        }
    }

    #[test]
    fn test_generate() {
        for _ in 0..1000 {
            let cpf = Cpf::generate();
            assert!(cpf.0.iter().all(|x| (0..=9).contains(x)));
            assert_eq!(
                &Cpf::compute_checksum(cpf.0.first_chunk().unwrap()),
                cpf.0.last_chunk().unwrap(),
                "generated invalid CPF: {cpf}"
            );
        }
    }

    #[test]
    fn test_compute_checksum() {
        for s in VALID_CPF_LIST {
            let cpf = s.parse::<Cpf>().unwrap();
            assert_eq!(
                Cpf::compute_checksum(cpf.0[..9].as_array().unwrap()),
                cpf.0[9..]
            );
        }
    }

    #[test]
    fn test_display() {
        for s in VALID_CPF_LIST {
            let s_ = common::remove_symbols(s, ".- ");
            assert_eq!(
                Cpf::from_str(s).unwrap().to_string(),
                format!("{}.{}.{}-{}", &s_[0..3], &s_[3..6], &s_[6..9], &s_[9..11])
            )
        }
    }
}
