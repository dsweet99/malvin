use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use crate::output::print_log_warning;
use crate::workspace_paths::malvin_logs_root;

pub use crate::log_gc_config::{load_logs_gc_config, LogsGcConfig};

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

pub fn prune_logs_before_run(work_dir: &Path) {
    let config = load_logs_gc_config(work_dir);
    let logs_root = malvin_logs_root(work_dir);
    if !logs_root.is_dir() {
        return;
    }
    let mut run_dirs = list_run_dirs(&logs_root);
    run_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let (removed, freed) = prune_run_dirs(&mut run_dirs, &config);
    if removed > 0 {
        eprintln!("[malvin] pruned {removed} run log(s) (~{} freed)", format_freed(freed));
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

pub(crate) fn prune_run_dirs(run_dirs: &mut Vec<PathBuf>, config: &LogsGcConfig) -> (usize, u64) {
    let mut removed = 0usize;
    let mut freed = 0u64;
    while needs_prune(run_dirs, config) {
        let mut pruned_one = false;
        let mut idx = run_dirs.len();
        while idx > 0 {
            idx -= 1;
            let path = run_dirs[idx].clone();
            let size = dir_size(&path);
            match std::fs::remove_dir_all(&path) {
                Ok(()) => {
                    run_dirs.remove(idx);
                    removed += 1;
                    freed = freed.saturating_add(size);
                    pruned_one = true;
                    break;
                }
                Err(e) => print_log_warning(&format!(
                    "could not prune log run {}: {e}",
                    path.display()
                )),
            }
        }
        if !pruned_one {
            break;
        }
    }
    (removed, freed)
}

pub(crate) fn needs_prune(run_dirs: &[PathBuf], config: &LogsGcConfig) -> bool {
    if run_dirs.is_empty() {
        return false;
    }
    if over_run_count(run_dirs, config.max_runs) {
        return true;
    }
    if over_byte_cap(run_dirs, config.max_bytes) {
        return true;
    }
    over_age_limit(run_dirs.last(), config.max_age_days)
}

pub(crate) const fn over_run_count(run_dirs: &[PathBuf], max_runs: u64) -> bool {
    max_runs > 0 && run_dirs.len() as u64 > max_runs
}

pub(crate) fn over_byte_cap(run_dirs: &[PathBuf], max_bytes: Option<u64>) -> bool {
    let Some(cap) = max_bytes else {
        return false;
    };
    run_dirs.iter().map(|p| dir_size(p)).sum::<u64>() > cap
}

pub(crate) fn over_age_limit(oldest: Option<&PathBuf>, max_age_days: u64) -> bool {
    if max_age_days == 0 {
        return false;
    }
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

pub(crate) fn format_freed(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * KIB;
    if bytes >= MIB {
        format!("{} MiB", bytes / MIB)
    } else if bytes >= KIB {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
#[path = "log_gc_tests.rs"]
mod log_gc_tests;
