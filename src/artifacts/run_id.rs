use chrono::Utc;
use rand::Rng;
use std::path::{Path, PathBuf};

pub fn create_run_dir(base_dir: Option<&Path>) -> std::io::Result<PathBuf> {
    let parent = base_dir.unwrap_or_else(|| Path::new("."));
    let run_root = parent.join("_malvin");
    std::fs::create_dir_all(&run_root)?;
    let identifier = build_identifier();
    let run_dir = run_root.join(&identifier);
    std::fs::create_dir(&run_dir)?;
    Ok(run_dir)
}

pub fn build_identifier() -> String {
    let stamp = Utc::now().format("%Y%m%d_%H%M%S");
    let token = random_alnum(8);
    format!("{stamp}_{token}")
}

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
