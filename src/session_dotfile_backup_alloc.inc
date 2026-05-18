use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[must_use]
pub(crate) fn malvin_home_dir() -> PathBuf {
    if let Some(h) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    if let Some(h) = std::env::var_os("USERPROFILE").filter(|s| !s.is_empty()) {
        return PathBuf::from(h);
    }
    std::env::temp_dir()
}

pub(crate) struct DotfileBackupLabels {
    pub mkdir: &'static str,
    pub collision: &'static str,
    pub restore: &'static str,
}

pub(crate) fn allocate_backup_dir(
    root: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
    labels: &DotfileBackupLabels,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", labels.mkdir))?;
    let mut tries = 0usize;
    while tries < 16 {
        let candidate = root.join(generate_id(tries));
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                tries += 1;
            }
            Err(err) => return Err(format!("{}: {err}", labels.mkdir)),
        }
    }
    Err(format!("{}: too many id collisions", labels.collision))
}

pub(crate) fn remove_if_exists(path: &Path, restore_label: &str) -> Result<(), String> {
    if path.exists() {
        let metadata = std::fs::metadata(path).map_err(|e| format!("{restore_label}: {e}"))?;
        if metadata.is_dir() {
            std::fs::remove_dir_all(path).map_err(|e| format!("{restore_label}: {e}"))?;
        } else {
            std::fs::remove_file(path).map_err(|e| format!("{restore_label}: {e}"))?;
        }
    }
    Ok(())
}

fn random_backup_id(_try_index: usize) -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..5)
        .map(|_| {
            let i = rng.gen_range(0..ALPHABET.len());
            ALPHABET[i] as char
        })
        .collect()
}
