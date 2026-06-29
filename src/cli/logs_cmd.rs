//! `malvin logs` — inspect and manually trigger run-log garbage collection.

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::log_gc::{
    format_freed, format_max_bytes_display, format_max_count_display, logs_bucket_status,
    prune_logs, sweep_empty_log_buckets, PruneOpts,
};
use crate::malvin_config_file::ensure_malvin_config_file;
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::workspace_paths::malvin_home_logs_root;

#[derive(Args, Debug)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub command: LogsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum LogsSubcommand {
    /// Report retention state for one workspace log bucket.
    Status(LogsStatusArgs),
    /// Prune old run logs without starting an agent session.
    Gc(LogsGcArgs),
}

#[derive(Args, Debug)]
pub struct LogsStatusArgs {
    /// Workspace directory (default: current directory).
    #[arg(long)]
    pub work_dir: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct LogsGcArgs {
    /// Workspace directory (default: current directory).
    #[arg(long)]
    pub work_dir: Option<PathBuf>,
    /// Report what would be deleted without removing directories.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Also remove empty hash buckets under `~/.malvin_home/logs/`.
    #[arg(long, default_value_t = false)]
    pub all_buckets: bool,
}

pub fn run_logs(args: LogsArgs) -> Result<(), String> {
    match args.command {
        LogsSubcommand::Status(status) => run_logs_status(status),
        LogsSubcommand::Gc(gc) => run_logs_gc(gc),
    }
}

fn resolve_work_dir(work_dir: Option<PathBuf>) -> Result<PathBuf, String> {
    work_dir.map_or_else(
        || std::env::current_dir().map_err(|e| format!("current directory: {e}")),
        Ok,
    )
}

fn run_logs_status(args: LogsStatusArgs) -> Result<(), String> {
    let work_dir = resolve_work_dir(args.work_dir)?;
    ensure_malvin_config_file(&work_dir)?;
    let status = logs_bucket_status(&work_dir);
    print_bucket_status(&status);
    Ok(())
}

fn run_logs_gc(args: LogsGcArgs) -> Result<(), String> {
    let work_dir = resolve_work_dir(args.work_dir)?;
    ensure_malvin_config_file(&work_dir)?;
    let result = prune_logs(
        &work_dir,
        PruneOpts {
            dry_run: args.dry_run,
            verbose: false,
        },
    );
    if args.dry_run {
        if result.would_remove > 0 {
            print_stdout_line(
                MALVIN_WHO,
                &format!("would prune {} run log(s)", result.would_remove),
            );
        }
    } else if result.removed > 0 {
        print_stdout_line(
            MALVIN_WHO,
            &format!(
                "pruned {} run log(s) (~{} freed)",
                result.removed,
                format_freed(result.freed)
            ),
        );
    }
    if args.all_buckets {
        let removed = sweep_empty_log_buckets(&malvin_home_logs_root());
        if removed > 0 {
            print_stdout_line(
                MALVIN_WHO,
                &format!("removed {removed} empty log bucket(s)"),
            );
        }
    }
    Ok(())
}

fn print_bucket_status(status: &crate::log_gc::LogsBucketStatus) {
    print_stdout_line(
        MALVIN_WHO,
        &format!("bucket: {}", status.bucket_path.display()),
    );
    print_stdout_line(MALVIN_WHO, &format!("run count: {}", status.run_count));
    print_stdout_line(
        MALVIN_WHO,
        &format!("total bytes: {}", format_freed(status.total_bytes)),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!(
            "oldest run: {}",
            status.oldest_run.as_deref().unwrap_or("(none)")
        ),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!(
            "newest run: {}",
            status.newest_run.as_deref().unwrap_or("(none)")
        ),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!(
            "max_count: {}",
            format_max_count_display(status.config.max_count)
        ),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!("max_age_days: {}", status.config.max_age_days),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!(
            "max_bytes: {}",
            format_max_bytes_display(status.config.max_bytes)
        ),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!("would byte cap prune: {}", status.would_byte_cap),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!("would count cap prune: {}", status.would_count_cap),
    );
    print_stdout_line(
        MALVIN_WHO,
        &format!("would age limit prune: {}", status.would_age_limit),
    );
}

#[cfg(test)]
#[path = "logs_cmd_kiss_cov_tests.rs"]
mod logs_cmd_kiss_cov_tests;
