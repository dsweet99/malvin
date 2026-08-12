use std::path::{Path, PathBuf};

use crate::log_gc_config::LogsGcConfig;
use crate::output::print_log_warning;
use crate::workspace_paths::read_work_dir_manifest;

use super::list_run_dirs;
use super::log_gc_prune::prune_run_dirs;

pub(crate) fn list_log_buckets(home_logs: &Path) -> Vec<PathBuf> {
    let mut buckets = Vec::new();
    let entries = match std::fs::read_dir(home_logs) {
        Ok(e) => e,
        Err(e) => {
            print_log_warning(&format!(
                "could not list {}: {e}",
                home_logs.display()
            ));
            return buckets;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            buckets.push(path);
        }
    }
    buckets
}

pub(crate) fn remove_bucket_if_empty(bucket: &Path, keep: Option<&Path>) {
    if keep.is_some_and(|k| k == bucket) {
        return;
    }
    let Ok(mut entries) = std::fs::read_dir(bucket) else {
        return;
    };
    if entries.next().is_some() {
        return;
    }
    if let Err(e) = std::fs::remove_dir(bucket) {
        print_log_warning(&format!(
            "could not remove empty log bucket {}: {e}",
            bucket.display()
        ));
    }
}

/// True when every run records a workspace path that no longer exists (ephemeral /tmp orphans).
pub(crate) fn bucket_is_ephemeral_orphan(runs: &[PathBuf]) -> bool {
    if runs.is_empty() {
        return false;
    }
    runs.iter().all(|run| {
        read_work_dir_manifest(run).is_some_and(|work| !work.exists())
    })
}

fn remove_orphan_bucket(bucket: &Path, runs: &[PathBuf]) -> (usize, u64) {
    let count = runs.len();
    let freed = crate::log_gc::dir_size(bucket);
    match std::fs::remove_dir_all(bucket) {
        Ok(()) => (count, freed),
        Err(e) => {
            print_log_warning(&format!(
                "could not remove orphan log bucket {}: {e}",
                bucket.display()
            ));
            (0, 0)
        }
    }
}

pub(crate) fn prune_all_log_buckets(
    home_logs: &Path,
    config: &LogsGcConfig,
    protect_run: Option<&Path>,
    keep_bucket: Option<&Path>,
) -> (usize, u64) {
    let mut removed = 0usize;
    let mut freed = 0u64;
    for bucket in list_log_buckets(home_logs) {
        let mut runs = list_run_dirs(&bucket);
        let keep = keep_bucket.is_some_and(|k| k == bucket.as_path());
        if !keep && bucket_is_ephemeral_orphan(&runs) {
            let (r, f) = remove_orphan_bucket(&bucket, &runs);
            removed = removed.saturating_add(r);
            freed = freed.saturating_add(f);
            continue;
        }
        runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
        let (r, f) = prune_run_dirs(&mut runs, config, protect_run);
        removed = removed.saturating_add(r);
        freed = freed.saturating_add(f);
        remove_bucket_if_empty(&bucket, keep_bucket);
    }
    (removed, freed)
}
