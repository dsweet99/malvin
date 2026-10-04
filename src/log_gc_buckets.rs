use std::path::{Path, PathBuf};

use crate::log_gc_config::LogsGcConfig;
use crate::output::print_log_warning;
use crate::workspace_paths::read_work_dir_manifest;

use super::log_gc_prune::{PruneTally, prune_run_dirs};
use super::{list_run_dirs, remove_tree_or_report, report_undeletable};

pub(crate) fn list_log_buckets(home_logs: &Path) -> Vec<PathBuf> {
    let mut buckets = Vec::new();
    let entries = match std::fs::read_dir(home_logs) {
        Ok(e) => e,
        Err(e) => {
            print_log_warning(&format!("could not list {}: {e}", home_logs.display()));
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

pub(crate) fn remove_bucket_if_empty(bucket: &Path, keep: Option<&Path>) -> bool {
    if keep.is_some_and(|k| k == bucket) {
        return true;
    }
    let Ok(mut entries) = std::fs::read_dir(bucket) else {
        return true;
    };
    if entries.next().is_some() {
        return true;
    }
    if let Err(e) = std::fs::remove_dir(bucket) {
        report_undeletable(bucket, &e);
        return false;
    }
    true
}

pub(crate) fn bucket_is_ephemeral_orphan(runs: &[PathBuf]) -> bool {
    if runs.is_empty() {
        return false;
    }
    runs.iter()
        .all(|run| read_work_dir_manifest(run).is_some_and(|work| !work.exists()))
}

fn remove_orphan_bucket(bucket: &Path, runs: &[PathBuf]) -> PruneTally {
    let freed = crate::log_gc::dir_size(bucket);
    if remove_tree_or_report(bucket) {
        PruneTally {
            removed: runs.len(),
            freed,
            aborted: false,
        }
    } else {
        PruneTally {
            aborted: true,
            ..PruneTally::default()
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
        let tally = if !keep && bucket_is_ephemeral_orphan(&runs) {
            remove_orphan_bucket(&bucket, &runs)
        } else {
            runs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            prune_run_dirs(&mut runs, config, protect_run)
        };
        removed = removed.saturating_add(tally.removed);
        freed = freed.saturating_add(tally.freed);
        if tally.aborted || !remove_bucket_if_empty(&bucket, keep_bucket) {
            break;
        }
    }
    (removed, freed)
}
