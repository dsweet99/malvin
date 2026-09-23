use std::fs;
use std::io::Write;
use std::path::Path;

use super::local_llm_ollama::process_alive;

pub(crate) fn try_acquire_lock(path: &Path) -> Result<bool, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    reclaim_stale_lock(path);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => {
            writeln!(file, "{}", std::process::id()).map_err(|e| e.to_string())?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(e) => Err(format!("lock {}: {e}", path.display())),
    }
}

pub(crate) fn release_lock(path: &Path) {
    let _ = fs::remove_file(path);
}

pub(crate) fn reclaim_stale_lock(path: &Path) {
    let Ok(body) = fs::read_to_string(path) else {
        return;
    };
    let Some(pid) = body.trim().parse::<u32>().ok() else {
        let _ = fs::remove_file(path);
        return;
    };
    if pid != std::process::id() && !process_alive(pid) {
        let _ = fs::remove_file(path);
    }
}

pub(crate) fn lock_holder_pid(path: &Path) -> Option<u32> {
    let body = fs::read_to_string(path).ok()?;
    let pid = body.trim().parse::<u32>().ok()?;
    process_alive(pid).then_some(pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_release_and_stale_reclaim() {
        let tmp = tempfile::tempdir().expect("tmp");
        let lock = tmp.path().join("m.lock");
        assert!(try_acquire_lock(&lock).expect("acq"));
        assert!(!try_acquire_lock(&lock).expect("busy"));
        release_lock(&lock);
        fs::write(&lock, "999999999\n").expect("stale");
        reclaim_stale_lock(&lock);
        assert!(!lock.is_file() || lock_holder_pid(&lock).is_none());
        assert!(try_acquire_lock(&lock).expect("after stale"));
        release_lock(&lock);
    }
}
