use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::output::{MALVIN_WHO, print_log_warning, print_stdout_line};
use crate::workspace_paths::malvin_logs_root;

pub use crate::log_gc_config::{load_logs_gc_config, LogsGcConfig};

#[path = "log_gc_format.rs"]
mod log_gc_format;
#[path = "log_gc_prune.rs"]
mod log_gc_prune;

pub(crate) use log_gc_format::{format_freed, format_max_bytes_display, format_max_count_display};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PruneOpts {
    pub dry_run: bool,
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PruneResult {
    pub removed: usize,
    pub freed: u64,
    pub would_remove: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogsBucketStatus {
    pub bucket_path: PathBuf,
    pub run_count: usize,
    pub total_bytes: u64,
    pub oldest_run: Option<String>,
    pub newest_run: Option<String>,
    pub config: LogsGcConfig,
    pub would_byte_cap: bool,
    pub would_count_cap: bool,
    pub would_age_limit: bool,
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
    let mut total = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            total = total.saturating_add(if p.is_dir() {
                dir_size_inner(&p)?
            } else {
                entry.metadata()?.len()
            });
        }
    } else if path.is_file() {
        total = path.metadata()?.len();
    }
    Ok(total)
}

pub fn prune_logs(work_dir: &Path, opts: PruneOpts) -> PruneResult {
    let config = load_logs_gc_config(work_dir);
    let logs_root = malvin_logs_root(work_dir);
    if !logs_root.is_dir() {
        return PruneResult {
            removed: 0,
            freed: 0,
            would_remove: 0,
        };
    }
    let mut run_dirs = list_run_dirs(&logs_root);
    run_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let (removed, freed, would_remove) = log_gc_prune::prune_run_dirs_with_opts(&mut run_dirs, &config, opts);
    PruneResult {
        removed,
        freed,
        would_remove,
    }
}

fn run_dir_display_names(run_dirs: &[PathBuf]) -> (Option<String>, Option<String>) {
    let dirname = |path: &PathBuf| {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
    };
    (
        run_dirs.last().and_then(dirname),
        run_dirs.first().and_then(dirname),
    )
}

pub fn logs_bucket_status(work_dir: &Path) -> LogsBucketStatus {
    let config = load_logs_gc_config(work_dir);
    let bucket_path = malvin_logs_root(work_dir);
    let mut run_dirs = if bucket_path.is_dir() {
        list_run_dirs(&bucket_path)
    } else {
        Vec::new()
    };
    run_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let total_bytes: u64 = run_dirs.iter().map(|p| dir_size(p)).sum();
    let (oldest_run, newest_run) = run_dir_display_names(&run_dirs);
    let (would_byte_cap, would_count_cap, would_age_limit) =
        log_gc_prune::policy_trigger_flags(&run_dirs, total_bytes, config);
    LogsBucketStatus {
        bucket_path,
        run_count: run_dirs.len(),
        total_bytes,
        oldest_run,
        newest_run,
        config,
        would_byte_cap,
        would_count_cap,
        would_age_limit,
    }
}

pub fn sweep_empty_log_buckets(home_logs_root: &Path) -> usize {
    let entries = match std::fs::read_dir(home_logs_root) {
        Ok(e) => e,
        Err(e) => {
            print_log_warning(&format!(
                "could not list {}: {e}",
                home_logs_root.display()
            ));
            return 0;
        }
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if list_run_dirs(&path).is_empty() && std::fs::remove_dir_all(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

pub fn prune_logs_before_run(work_dir: &Path) {
    let result = prune_logs(work_dir, PruneOpts::default());
    if result.removed > 0 {
        print_stdout_line(
            MALVIN_WHO,
            &format!(
                "pruned {} run log(s) (~{} freed)",
                result.removed,
                format_freed(result.freed)
            ),
        );
    }
}

pub(crate) fn list_run_dirs(logs_root: &Path) -> Vec<PathBuf> {
    let mut runs = Vec::new();
    let entries = match std::fs::read_dir(logs_root) {
        Ok(e) => e,
        Err(e) => {
            print_log_warning(&format!(
                "could not list {}: {e}",
                logs_root.display()
            ));
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
