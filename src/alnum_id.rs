use rand::Rng;

#[must_use]
pub fn random_alnum(len: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let i = rng.gen_range(0..ALPHABET.len());
            ALPHABET[i] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn random_alnum_len_matches_request() {
        assert_eq!(super::random_alnum(12).len(), 12);
        assert_eq!(super::random_alnum(0).len(), 0);
    }

    #[test]
    fn random_alnum_charset_fuzz_samples() {
        for len in [1usize, 8, 32, 64] {
            let s = super::random_alnum(len);
            assert_eq!(s.len(), len);
            assert!(
                s.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
                "non-alnum char in {s:?}"
            );
            assert!(!s.chars().any(|c| c.is_ascii_uppercase()));
        }
    }
}
