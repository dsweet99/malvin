//! External kiss witnesses for [`super`] (must be `*_tests.rs` for kiss).

#[test]
fn kiss_cov_logs_args_parse_status_and_gc() {
    use clap::Parser;

    use crate::cli::args::{Cli, Commands, LogsArgs};
    use super::{LogsGcArgs, LogsStatusArgs, LogsSubcommand};

    let cli = Cli::try_parse_from(["malvin", "logs", "status"]).expect("parse status");
    assert!(matches!(
        cli.command,
        Some(Commands::Logs(LogsArgs {
            command: LogsSubcommand::Status(LogsStatusArgs { work_dir: None }),
        }))
    ));

    let gc_cli = Cli::try_parse_from(["malvin", "logs", "gc", "--dry-run"]).expect("parse gc");
    assert!(matches!(
        gc_cli.command,
        Some(Commands::Logs(LogsArgs {
            command: LogsSubcommand::Gc(LogsGcArgs {
                dry_run: true,
                all_buckets: false,
                work_dir: None,
            }),
        }))
    ));
}

#[test]
fn kiss_cov_logs_cmd_run_status() {
    use super::{run_logs, LogsArgs, LogsStatusArgs, LogsSubcommand};

    crate::test_utils::with_isolated_home(|work| {
        std::fs::create_dir_all(
            crate::workspace_paths::malvin_logs_root(work).join("20260101_000000_aaaaaaa1"),
        )
        .expect("seed run");
        run_logs(LogsArgs {
            command: LogsSubcommand::Status(LogsStatusArgs {
                work_dir: Some(work.to_path_buf()),
            }),
        })
        .expect("logs status");
    });
}

#[test]
fn kiss_cov_logs_cmd_run_gc_dry_run() {
    use super::{run_logs, LogsArgs, LogsGcArgs, LogsSubcommand};

    crate::test_utils::with_isolated_home(|work| {
        std::fs::create_dir_all(
            crate::workspace_paths::malvin_logs_root(work).join("20260101_000000_aaaaaaa1"),
        )
        .expect("seed run");
        run_logs(LogsArgs {
            command: LogsSubcommand::Gc(LogsGcArgs {
                work_dir: Some(work.to_path_buf()),
                dry_run: true,
                all_buckets: false,
            }),
        })
        .expect("logs gc dry-run");
    });
}

#[test]
fn kiss_cov_logs_cmd_all_buckets_sweep() {
    use super::{run_logs, LogsArgs, LogsGcArgs, LogsSubcommand};

    crate::test_utils::with_isolated_home(|work| {
        let empty_bucket = crate::workspace_paths::malvin_home_logs_root().join("emptybucket0");
        std::fs::create_dir_all(&empty_bucket).expect("mkdir empty bucket");
        run_logs(LogsArgs {
            command: LogsSubcommand::Gc(LogsGcArgs {
                work_dir: Some(work.to_path_buf()),
                dry_run: false,
                all_buckets: true,
            }),
        })
        .expect("logs gc all-buckets");
        assert!(!empty_bucket.exists());
    });
}

#[test]
fn kiss_cov_logs_cmd_doc_embedded() {
    use clap::Parser;

    use crate::cli::command_docs::command_doc_markdown;
    use crate::cli::args::{Cli, Commands, LogsArgs};
    use super::{LogsStatusArgs, LogsSubcommand};

    let md = command_doc_markdown(&Commands::Logs(LogsArgs {
        command: LogsSubcommand::Status(LogsStatusArgs { work_dir: None }),
    }));
    assert!(md.starts_with("# malvin logs"));
    let cli = Cli::try_parse_from(["malvin", "logs", "status", "--doc"]).expect("parse");
    assert!(cli.shared.doc);
}
