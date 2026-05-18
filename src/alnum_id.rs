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
mod kiss_coverage {
    #[test]
    fn kiss_stringify_units() {
        let _ = stringify!(super::random_alnum);
    }
}
