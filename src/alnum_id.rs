//! Short random alphanumeric ids (leaf; no malvin internals).

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
    }
}
