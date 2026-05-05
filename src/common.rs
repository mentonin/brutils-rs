pub fn modulo11_sum<'a, I>(digits: I) -> u8
where
    I: IntoIterator<Item = &'a u8, IntoIter: DoubleEndedIterator>,
{
    digits
        .into_iter()
        .rev()
        .zip(1..)
        .map(|(d, i)| (d % 11, i % 11))
        .fold(0, |acc, (d, i)| (acc + d * i) % 11)
}

pub fn modulo11_gen<'a, I>(base_digits: I) -> u8
where
    I: IntoIterator<Item = &'a u8, IntoIter: DoubleEndedIterator>,
{
    (11 - modulo11_sum(base_digits.into_iter().chain(&[0u8]))) % 11
}

pub fn remove_symbols(dirty: &str, symbols: &str) -> String {
    dirty.chars().filter(|&c| !symbols.contains(c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulo11_gen() {
        assert_eq!(modulo11_gen(&[0, 0, 0, 0, 0, 0, 0, 0, 0]), 0);
        assert_eq!(modulo11_gen(&[5, 2, 5, 1, 3, 1, 2, 7, 7]), 6);
        assert_eq!(modulo11_gen(&[2, 5, 1, 3, 1, 2, 7, 7, 6]), 5);
        assert_eq!(modulo11_gen(&[5, 2, 5, 9, 9, 9, 2, 7, 7]), 6);
        assert_eq!(modulo11_gen(&[2, 5, 9, 9, 9, 2, 7, 7, 6]), 5);
    }

    #[test]
    fn test_modulo11_sum() {
        assert_eq!(modulo11_sum(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]), 0);
        assert_eq!(modulo11_sum(&[5, 2, 5, 1, 3, 1, 2, 7, 7, 0]), 5);
        assert_eq!(modulo11_sum(&[2, 5, 1, 3, 1, 2, 7, 7, 6, 0]), 6);
        assert_eq!(modulo11_sum(&[5, 2, 5, 9, 9, 9, 2, 7, 7, 0]), 5);
        assert_eq!(modulo11_sum(&[2, 5, 9, 9, 9, 2, 7, 7, 6, 0]), 6);
    }
}
