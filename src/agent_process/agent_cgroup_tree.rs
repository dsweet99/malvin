use std::path::{Path, PathBuf};

use tracing::warn;

pub const LEAF_NAME: &str = "agent";

#[must_use]
pub fn child_cgroup_dirs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter(is_dir_entry)
        .map(|entry| entry.path())
        .collect()
}

fn is_dir_entry(entry: &std::fs::DirEntry) -> bool {
    entry.file_type().is_ok_and(|t| t.is_dir())
}

pub fn remove_cgroup_tree(dir: &Path) {
    for child in child_cgroup_dirs(dir) {
        remove_cgroup_tree(&child);
    }
    match std::fs::remove_dir(dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => warn!(error = %e, dir = %dir.display(), "cannot remove nested agent cgroup"),
    }
}
