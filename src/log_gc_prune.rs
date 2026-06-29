use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Utc};

use super::{dir_size, run_dir_timestamp, PruneOpts};
use crate::log_gc_config::LogsGcConfig;
use crate::output::{MALVIN_WHO, print_log_warning, print_stdout_line};

#[allow(dead_code)]
pub(crate) fn prune_run_dirs(run_dirs: &mut Vec<PathBuf>, config: &LogsGcConfig) -> (usize, u64) {
    let (removed, freed, _) =
        prune_run_dirs_with_opts(run_dirs, config, PruneOpts::default());
    (removed, freed)
}

pub(crate) fn prune_run_dirs_with_opts(
    run_dirs: &mut Vec<PathBuf>,
    config: &LogsGcConfig,
    opts: PruneOpts,
) -> (usize, u64, usize) {
    if run_dirs.is_empty() {
        return (0, 0, 0);
    }
    let mut sizes: Vec<u64> = run_dirs.iter().map(|p| dir_size(p)).collect();
    let mut total_bytes: u64 = sizes.iter().copied().sum();
    let mut removed = 0usize;
    let mut freed = 0u64;
    let mut would_remove = 0usize;

    while needs_prune(run_dirs, total_bytes, config) {
        match remove_oldest_run(run_dirs, &mut sizes, &mut total_bytes, opts) {
            PruneStepOutcome::Simulated => would_remove += 1,
            PruneStepOutcome::Removed(size) => {
                removed += 1;
                freed = freed.saturating_add(size);
            }
            PruneStepOutcome::Blocked => break,
        }
    }
    (removed, freed, would_remove)
}

pub(crate) fn needs_prune(
    run_dirs: &[PathBuf],
    total_bytes: u64,
    config: &LogsGcConfig,
) -> bool {
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

pub(crate) fn over_count_cap(run_count: usize, max_count: u64) -> bool {
    max_count > 0 && u64::try_from(run_count).unwrap_or(u64::MAX) > max_count
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

fn remove_oldest_run(
    run_dirs: &mut Vec<PathBuf>,
    sizes: &mut Vec<u64>,
    total_bytes: &mut u64,
    opts: PruneOpts,
) -> PruneStepOutcome {
    let mut idx = run_dirs.len();
    while idx > 0 {
        idx -= 1;
        let path = run_dirs[idx].clone();
        let size = sizes[idx];
        if opts.dry_run {
            run_dirs.remove(idx);
            sizes.remove(idx);
            *total_bytes = total_bytes.saturating_sub(size);
            if opts.verbose {
                print_stdout_line(MALVIN_WHO, &format!("would prune {}", path.display()));
            }
            return PruneStepOutcome::Simulated;
        }
        match std::fs::remove_dir_all(&path) {
            Ok(()) => {
                run_dirs.remove(idx);
                sizes.remove(idx);
                *total_bytes = total_bytes.saturating_sub(size);
                if opts.verbose {
                    print_stdout_line(MALVIN_WHO, &format!("pruned {}", path.display()));
                }
                return PruneStepOutcome::Removed(size);
            }
            Err(e) => print_log_warning(&format!(
                "could not prune log run {}: {e}",
                path.display()
            )),
        }
    }
    PruneStepOutcome::Blocked
}

enum PruneStepOutcome {
    Simulated,
    Removed(u64),
    Blocked,
}

pub(crate) fn policy_trigger_flags(
    run_dirs: &[PathBuf],
    total_bytes: u64,
    config: LogsGcConfig,
) -> (bool, bool, bool) {
    (
        over_byte_cap(total_bytes, config.max_bytes),
        over_count_cap(run_dirs.len(), config.max_count),
        over_age_limit(run_dirs.last(), config.max_age_days),
    )
}
