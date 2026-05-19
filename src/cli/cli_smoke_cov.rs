//! Behavioral smoke tests for CLI helpers.

use super::entrypoint::{require_kiss_for_cli_command, try_tokio_runtime};
use super::{Cli, Commands};

#[test]
fn smoke_has_source_files_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(!crate::source_detect::has_source_files(tmp.path()));
}

#[test]
fn smoke_has_source_files_detects_rs() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("x.rs"), "").unwrap();
    assert!(crate::source_detect::has_source_files(tmp.path()));
}

#[test]
fn smoke_merge_acp_and_timing_results() {
    use crate::acp_post_run::merge_acp_and_timing_results;
    assert_eq!(merge_acp_and_timing_results(Ok(()), Ok(())), Ok(()));
    assert_eq!(
        merge_acp_and_timing_results(Err("acp".into()), Err(std::io::Error::other("io"))),
        Err("acp".into())
    );
}

#[test]
fn smoke_prefer_primary_over_secondary() {
    use crate::acp_post_run::prefer_primary_over_secondary;
    assert_eq!(prefer_primary_over_secondary(Ok(()), Ok(()), "x"), Ok(()));
    assert_eq!(
        prefer_primary_over_secondary(Ok(()), Err("b".into()), "x"),
        Err("b".into())
    );
}

fn empty_session_backups(work: &std::path::Path) -> crate::artifacts::SessionDotfileBackups {
    crate::artifacts::SessionDotfileBackups::from_parts(
        crate::artifacts::backup_workspace_kissconfig_if_present(work).unwrap(),
        crate::artifacts::backup_workspace_malvin_checks_if_present(work).unwrap(),
        crate::artifacts::backup_workspace_kissignore_if_present(work).unwrap(),
    )
}

#[test]
fn smoke_merge_acp_with_workspace_session_restore() {
    let work = tempfile::tempdir().unwrap();
    let backups = empty_session_backups(work.path());
    assert!(
        crate::acp_post_run::merge_acp_with_workspace_session_restore(Ok(()), work.path(), &backups)
            .is_ok()
    );
}

#[test]
fn smoke_merge_acp_with_workspace_session_restore_and_check_abort_no_result_file() {
    let work = tempfile::tempdir().unwrap();
    let missing = work.path().join("no_such_result.md");
    let backups = empty_session_backups(work.path());
    assert!(
        crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
            Ok(()),
            work.path(),
            &backups,
            &missing,
        )
        .is_ok()
    );
}

#[test]
fn smoke_agent_io_options_maps_flags() {
    use super::{AgentStdoutTeeFlags, WorkflowCliOptions, agent_io_options};
    let shared = super::SharedOpts {
        model: "m".into(),
        no_force: false,
        no_tee: true,
        no_markdown: false,
        verbose: false,
    };
    let io = agent_io_options(
        &shared,
        WorkflowCliOptions {
            force: true,
            run_learn: true,
        },
        AgentStdoutTeeFlags {
            emit_stdout_markdown: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
        },
    );
    assert!(io.force);
    assert!(io.no_tee);
    assert!(io.raw_output);
    assert!(!io.show_thoughts_on_stdout);
    assert!(io.emit_stdout_markdown);
    assert!(!io.log_full_outgoing_prompts);
}

#[test]
fn smoke_cli_parse_models_subcommand() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "models"]).unwrap();
    assert!(matches!(cli.command, Commands::Models(_)));
}

#[test]
fn smoke_try_tokio_runtime_builds_multi_thread() {
    let _rt = try_tokio_runtime().expect("tokio runtime");
}

#[test]
fn smoke_require_kiss_for_cli_command_models_does_not_require_kiss_bin() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "models"]).unwrap();
    assert!(require_kiss_for_cli_command(&cli.command).is_ok());
}
