use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::workspace::workspace_paths::{malvin_home_logs_root, malvin_logs_root};

pub use crate::config::log_gc_config::load_logs_gc_config;

#[path = "log_gc_buckets.rs"]
mod log_gc_buckets;
#[path = "log_gc_format.rs"]
mod log_gc_format;
#[path = "log_gc_fs.rs"]
mod log_gc_fs;
#[path = "log_gc_prune.rs"]
mod log_gc_prune;

pub(crate) use log_gc_format::format_freed;
#[cfg(test)]
pub(crate) use log_gc_fs::{
    dir_size, dir_size_inner, is_run_log_dir_name, list_run_dirs, remove_tree, run_dir_timestamp,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PruneResult {
    pub removed: usize,
    pub freed: u64,
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

#[cfg(test)]
#[path = "log_gc_tests.rs"]
mod log_gc_tests;

#[cfg(test)]
#[path = "log_gc_v1_tests.rs"]
mod log_gc_v1_tests;

#[cfg(test)]
#[path = "log_gc_cross_tests.rs"]
mod log_gc_cross_tests;
