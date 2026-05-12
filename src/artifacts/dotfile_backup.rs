use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) struct DotfileBackupLabels {
    pub mkdir: &'static str,
    pub collision: &'static str,
    pub restore: &'static str,
}

pub(super) fn allocate_backup_dir(
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

pub(super) fn remove_if_exists(path: &Path, restore_label: &str) -> Result<(), String> {
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

#[cfg(test)]
pub(super) mod test_support {
    #![allow(unsafe_code)]

    use std::path::Path;

    pub(in crate::artifacts) fn with_isolated_home<F>(f: F)
    where
        F: FnOnce(&Path),
    {
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let old_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(&work).unwrap();
        f(&work);
        unsafe {
            match old_home {
                Some(h) => std::env::set_var("HOME", h),
                None => std::env::remove_var("HOME"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_stringify_dotfile_backup_units() {
        let _ = stringify!(super::DotfileBackupLabels);
        let _ = stringify!(super::allocate_backup_dir);
        let _ = stringify!(super::remove_if_exists);
        let _ = stringify!(super::test_support::with_isolated_home);
    }
}
