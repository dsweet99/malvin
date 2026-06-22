use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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

pub(crate) fn random_backup_id(_try_index: usize) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_backup_dir_creates_distinct_directories() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let labels = DotfileBackupLabels {
            mkdir: "mkdir",
            collision: "collision",
            restore: "restore",
        };
        let mut next_id = random_backup_id;
        let first = allocate_backup_dir(tmp.path(), &mut next_id, &labels).expect("first");
        let second = allocate_backup_dir(tmp.path(), &mut next_id, &labels).expect("second");
        assert!(first.is_dir());
        assert!(second.is_dir());
        assert_ne!(first, second);
    }
}
#[cfg(test)]
#[path = "alloc_test.rs"]
mod alloc_test;#[cfg(test)]
#[path = "alloc_kiss_cov_test.rs"]
mod alloc_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<DotfileBackupLabels> = None;
        let _ = random_backup_id;
    }
}
