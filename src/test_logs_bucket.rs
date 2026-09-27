use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::{Duration, SystemTime};

const LEGACY_FIXTURE_NAMES: [&str; 4] = ["aaabbbcc", "aaabbbcd", "stuck", "hand_notes"];
const LEGACY_MIN_AGE: Duration = Duration::from_secs(600);
static LEGACY_SWEEP: Once = Once::new();

/// Workspace dir plus its hash bucket under the malvin home logs root, at stable paths.
/// Both are removed on creation (leftovers from an earlier run) and on drop.
/// Holds `test_env_lock` so `HOME` (and thus the logs root) cannot change while it lives.
pub struct TestLogsBucket {
    work: PathBuf,
    bucket: PathBuf,
    _env_lock: std::sync::MutexGuard<'static, ()>,
}

impl TestLogsBucket {
    pub fn new(name: &str) -> Self {
        let env_lock = super::test_env_lock();
        LEGACY_SWEEP.call_once(sweep_legacy_buckets);
        let work = std::env::temp_dir().join(format!("malvin-test-logs-{name}"));
        remove_tree(&work);
        std::fs::create_dir_all(&work).expect("mkdir test work dir");
        let bucket = crate::workspace_paths::malvin_logs_root(&work);
        remove_tree(&bucket);
        Self {
            work,
            bucket,
            _env_lock: env_lock,
        }
    }

    pub fn work(&self) -> &Path {
        &self.work
    }

    pub fn bucket(&self) -> &Path {
        &self.bucket
    }
}

impl Drop for TestLogsBucket {
    fn drop(&mut self) {
        remove_tree(&self.bucket);
        remove_tree(&self.work);
    }
}

fn remove_tree(path: &Path) {
    #[cfg(unix)]
    restore_child_permissions(path);
    let _ = std::fs::remove_dir_all(path);
}

#[cfg(unix)]
fn restore_child_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            let _ = std::fs::set_permissions(entry.path(), std::fs::Permissions::from_mode(0o700));
        }
    }
}

fn sweep_legacy_buckets() {
    let Ok(entries) = std::fs::read_dir(crate::workspace_paths::malvin_home_logs_root()) else {
        return;
    };
    for bucket in entries.flatten().map(|e| e.path()) {
        if is_hash_bucket_name(&bucket) && is_stale(&bucket) && holds_only_fixtures(&bucket) {
            let _ = std::fs::remove_dir_all(&bucket);
        }
    }
}

fn is_hash_bucket_name(bucket: &Path) -> bool {
    bucket.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        n.len() == 16 && n.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

fn is_stale(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age >= LEGACY_MIN_AGE)
}

fn holds_only_fixtures(bucket: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(bucket) else {
        return false;
    };
    let entries: Vec<_> = entries.collect();
    !entries.is_empty()
        && entries
            .iter()
            .all(|e| e.as_ref().is_ok_and(|e| is_fixture_dir(&e.path())))
}

fn is_fixture_dir(dir: &Path) -> bool {
    let named = dir.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        LEGACY_FIXTURE_NAMES.contains(&n) || crate::log_gc::is_run_log_dir_name(n)
    });
    named
        && std::fs::read_dir(dir).is_ok_and(|mut it| {
            it.all(|e| e.is_ok_and(|e| e.file_name() == "payload" && e.path().is_file()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_and_work_are_removed_on_drop_and_on_recreate() {
        let first = TestLogsBucket::new("self-test");
        let (work, bucket) = (first.work().to_path_buf(), first.bucket().to_path_buf());
        drop(first);
        std::fs::create_dir_all(bucket.join("stuck")).expect("mkdir leftover bucket");
        std::fs::create_dir_all(work.join("junk")).expect("mkdir leftover work");
        let second = TestLogsBucket::new("self-test");
        assert!(!work.join("junk").exists(), "leftover work dir must be removed at start");
        assert_eq!(second.bucket(), bucket.as_path());
        assert!(!bucket.exists(), "leftover bucket must be removed at start");
        drop(second);
        assert!(!work.exists() && !bucket.exists());
    }

    #[test]
    fn fixture_detection_accepts_only_fixture_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bucket = tmp.path().join("0123456789abcdef");
        std::fs::create_dir_all(bucket.join("aaabbbcc")).expect("mkdir");
        std::fs::create_dir_all(bucket.join("20260103_000000_ccccccc3")).expect("mkdir");
        std::fs::write(bucket.join("20260103_000000_ccccccc3/payload"), b"x").expect("write");
        assert!(is_hash_bucket_name(&bucket));
        assert!(holds_only_fixtures(&bucket));
        assert!(!is_stale(&bucket), "fresh buckets are never swept");
        std::fs::write(bucket.join("20260103_000000_ccccccc3/command.log"), b"x").expect("write");
        assert!(!holds_only_fixtures(&bucket), "real run content keeps the bucket");
        assert!(!is_hash_bucket_name(tmp.path()));
        assert!(!holds_only_fixtures(&tmp.path().join("missing")));
    }
}
