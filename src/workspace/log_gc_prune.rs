use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};

use super::{dir_size, remove_tree_or_report, run_dir_timestamp};
use crate::config::log_gc_config::LogsGcConfig;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PruneTally {
    pub removed: usize,
    pub freed: u64,
    pub aborted: bool,
}

enum RemoveStep {
    Removed(u64),
    NothingLeft,
    Undeletable,
}

pub(crate) fn prune_run_dirs(
    run_dirs: &mut Vec<PathBuf>,
    config: &LogsGcConfig,
    protect: Option<&Path>,
) -> PruneTally {
    let mut tally = PruneTally::default();
    if run_dirs.is_empty() {
        return tally;
    }
    let count_or_age = over_count_cap(run_dirs.len(), config.max_count)
        || over_age_limit(run_dirs.last(), config.max_age_days);
    if !count_or_age && config.max_bytes.is_none() {
        return tally;
    }
    let mut sizes: Vec<u64> = run_dirs.iter().map(|p| dir_size(p)).collect();
    let mut total_bytes: u64 = sizes.iter().copied().sum();
    if !count_or_age && !over_byte_cap(total_bytes, config.max_bytes) {
        return tally;
    }
    while needs_prune(run_dirs, total_bytes, config) {
        match remove_oldest_run(run_dirs, &mut sizes, &mut total_bytes, protect) {
            RemoveStep::Removed(size) => {
                tally.removed += 1;
                tally.freed = tally.freed.saturating_add(size);
            }
            RemoveStep::NothingLeft => break,
            RemoveStep::Undeletable => {
                tally.aborted = true;
                break;
            }
        }
    }
    tally
}

pub(crate) fn needs_prune(run_dirs: &[PathBuf], total_bytes: u64, config: &LogsGcConfig) -> bool {
    if run_dirs.is_empty() {
        return false;
    }
    if over_byte_cap(total_bytes, config.max_bytes) {
        return true;
    }
    if over_count_cap(run_dirs.len(), config.max_count) {
        return true;
    }
    over_age_limit(run_dirs.last(), config.max_age_days)
}

pub(crate) const fn over_byte_cap(total_bytes: u64, max_bytes: Option<u64>) -> bool {
    let Some(cap) = max_bytes else {
        return false;
    };
    total_bytes > cap
}

pub(crate) fn over_count_cap(run_count: usize, max_count: Option<u64>) -> bool {
    let Some(cap) = max_count else {
        return false;
    };
    u64::try_from(run_count).unwrap_or(u64::MAX) > cap
}

pub(crate) fn over_age_limit(oldest: Option<&PathBuf>, max_age_days: Option<u64>) -> bool {
    let Some(max_age_days) = max_age_days else {
        return false;
    };
    let Some(path) = oldest else {
        return false;
    };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let ts = run_dir_timestamp(name).or_else(|| mtime_as_utc(path));
    let cutoff = Utc::now() - Duration::days(i64::try_from(max_age_days).unwrap_or(i64::MAX));
    ts.is_some_and(|t| t < cutoff)
}

pub(crate) fn mtime_as_utc(path: &Path) -> Option<DateTime<Utc>> {
    let modified = path.metadata().ok()?.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
}

fn remove_oldest_run(
    run_dirs: &mut Vec<PathBuf>,
    sizes: &mut Vec<u64>,
    total_bytes: &mut u64,
    protect: Option<&Path>,
) -> RemoveStep {
    let Some(idx) = run_dirs
        .iter()
        .rposition(|path| protect.is_none_or(|p| p != path.as_path()))
    else {
        return RemoveStep::NothingLeft;
    };
    if !remove_tree_or_report(&run_dirs[idx]) {
        return RemoveStep::Undeletable;
    }
    run_dirs.remove(idx);
    let size = sizes.remove(idx);
    *total_bytes = total_bytes.saturating_sub(size);
    RemoveStep::Removed(size)
}
