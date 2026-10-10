use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::output::{MALVIN_WHO, print_log_error, print_log_warning, print_stdout_line};
use crate::workspace::workspace_paths::{malvin_home_logs_root, malvin_logs_root};

pub use crate::config::log_gc_config::load_logs_gc_config;

#[path = "log_gc_buckets.rs"]
mod log_gc_buckets;
#[path = "log_gc_format.rs"]
mod log_gc_format;
#[path = "log_gc_prune.rs"]
mod log_gc_prune;

pub(crate) use log_gc_format::format_freed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PruneResult {
    pub removed: usize,
    pub freed: u64,
}

pub fn run_dir_timestamp(name: &str) -> Option<DateTime<Utc>> {
    if name.len() < 15 {
        return None;
    }
    let stamp = &name[..15];
    let naive = NaiveDateTime::parse_from_str(stamp, "%Y%m%d_%H%M%S").ok()?;
    Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

pub(crate) fn is_run_log_dir_name(name: &str) -> bool {
    const STAMP_LEN: usize = 15;
    const TOKEN_LEN: usize = 8;
    if name.len() != STAMP_LEN + 1 + TOKEN_LEN {
        return false;
    }
    if name.as_bytes()[STAMP_LEN] != b'_' {
        return false;
    }
    if run_dir_timestamp(name).is_none() {
        return false;
    }
    name[STAMP_LEN + 1..]
        .bytes()
        .all(|b| b.is_ascii_alphanumeric())
}

pub fn dir_size(path: &Path) -> u64 {
    dir_size_inner(path).unwrap_or(0)
}

pub(crate) fn dir_size_inner(path: &Path) -> std::io::Result<u64> {
    let meta = std::fs::symlink_metadata(path)?;
    let ft = meta.file_type();
    if ft.is_symlink() {
        return Ok(meta.len());
    }
    if !ft.is_dir() {
        return Ok(if ft.is_file() { meta.len() } else { 0 });
    }
    let mut total = 0u64;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        total = total.saturating_add(dir_size_inner(&entry.path())?);
    }
    Ok(total)
}

pub(crate) fn remove_tree(path: &Path) -> Result<(), (PathBuf, std::io::Error)> {
    let fail = |e: std::io::Error| (path.to_path_buf(), e);
    if std::fs::symlink_metadata(path).map_err(fail)?.is_dir() {
        for entry in std::fs::read_dir(path).map_err(fail)? {
            remove_tree(&entry.map_err(fail)?.path())?;
        }
        std::fs::remove_dir(path).map_err(fail)
    } else {
        std::fs::remove_file(path).map_err(fail)
    }
}

pub(crate) fn report_undeletable(path: &Path, e: &std::io::Error) {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    print_log_error(&format!(
        "log GC skipped the rest of this log folder: could not delete {}: {e}",
        abs.display()
    ));
}

pub(crate) fn remove_tree_or_report(path: &Path) -> bool {
    match remove_tree(path) {
        Ok(()) => true,
        Err((stuck, e)) => {
            report_undeletable(&stuck, &e);
            false
        }
    }
}

fn prune_logs(work_dir: &Path, protect_run: Option<&Path>) -> PruneResult {
    let config = load_logs_gc_config(work_dir);
    let home_logs = malvin_home_logs_root();
    if !home_logs.is_dir() {
        return PruneResult {
            removed: 0,
            freed: 0,
        };
    }
    let keep_bucket = malvin_logs_root(work_dir);
    let (removed, freed) = log_gc_buckets::prune_all_log_buckets(
        &home_logs,
        &config,
        protect_run,
        Some(keep_bucket.as_path()),
    );
    PruneResult { removed, freed }
}

fn emit_prune_result(result: PruneResult) {
    if result.removed > 0 {
        print_stdout_line(
            MALVIN_WHO,
            &format!(
                "Pruned {} run log(s) (~{} freed)",
                result.removed,
                format_freed(result.freed)
            ),
        );
    }
}

pub fn prune_logs_after_run_created(work_dir: &Path, protect_run: &Path) {
    emit_prune_result(prune_logs(work_dir, Some(protect_run)));
}

pub(crate) fn list_run_dirs(logs_root: &Path) -> Vec<PathBuf> {
    let mut runs = Vec::new();
    let entries = match std::fs::read_dir(logs_root) {
        Ok(e) => e,
        Err(e) => {
            print_log_warning(&format!("could not list {}: {e}", logs_root.display()));
            return runs;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_run = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(is_run_log_dir_name);
        if path.is_dir() && is_run {
            runs.push(path);
        }
    }
    runs
}

#[cfg(test)]
#[path = "log_gc_tests.rs"]
mod log_gc_tests;

#[cfg(test)]
#[path = "log_gc_v1_tests.rs"]
mod log_gc_v1_tests;

#[cfg(test)]
#[path = "log_gc_cross_tests.rs"]
mod log_gc_cross_tests;
