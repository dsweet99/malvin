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

#[test]
fn smoke_merge_acp_with_workspace_session_restore() {
    let work = tempfile::tempdir().unwrap();
    let backups = crate::test_utils::empty_session_dotfile_backups(work.path());
    assert!(
        crate::acp_post_run::merge_acp_with_workspace_session_restore(
            Ok(()),
            work.path(),
            &backups
        )
        .is_ok()
    );
}

#[test]
fn smoke_merge_acp_with_workspace_session_restore_and_check_abort_no_result_file() {
    let work = tempfile::tempdir().unwrap();
    let missing = work.path().join("no_such_result.md");
    let backups = crate::test_utils::empty_session_dotfile_backups(work.path());
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
        doc: false,
        sandbox: false,
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
    assert!(matches!(cli.command, Some(Commands::Models(_))));
}

#[test]
fn smoke_try_tokio_runtime_builds_multi_thread() {
    let _rt = try_tokio_runtime().expect("tokio runtime");
}

#[test]
fn smoke_require_kiss_for_cli_command_models_does_not_require_kiss_bin() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "models"]).unwrap();
    let cmd = cli.command.as_ref().expect("subcommand");
    assert!(require_kiss_for_cli_command(cmd).is_ok());
}

#[test]
fn smoke_tidy_effective_max_loops() {
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(0), 1);
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(3), 3);
}

#[test]
fn smoke_parse_languages() {
    let langs = crate::init_cmd::parse_languages(&["rust".into(), "python".into()]).expect("parse");
    assert_eq!(langs.len(), 2);
}

#[test]
fn smoke_emit_command_line_writes_log() {
    use std::path::Path;
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    super::run_emit::emit_command_line(Path::new(&run_dir), false).expect("emit");
    assert!(run_dir.join("command.log").is_file());
}

#[test]
fn smoke_cli_parse_plan_subcommand() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "plan", "hello"]).expect("parse");
    assert!(matches!(cli.command, Some(Commands::Plan(_))));
}

#[test]
fn smoke_format_logs_dir_under_run_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let logs = crate::format_logs_dir(&run_dir).expect("logs dir");
    assert!(logs.contains("run"));
}

#[test]
fn smoke_cli_parse_init_subcommand() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "init", "rust"]).expect("parse");
    assert!(matches!(cli.command, Some(Commands::Init(_))));
}

#[test]
fn smoke_run_emit_echo_primary_noop_when_not_plain() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "plan").expect("write plan");
    super::run_emit::echo_primary_to_stdout(&plan, false, "malvin").expect("echo");
}

#[test]
fn smoke_require_kiss_allows_do_without_kiss_on_path() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "do", "task"]).expect("parse");
    let cmd = cli.command.as_ref().expect("subcommand");
    assert!(require_kiss_for_cli_command(cmd).is_ok());
}

#[test]
fn smoke_print_command_error_writes_run_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    crate::cli::error_run_log::set_command_error_run_dir(Some(run_dir.clone()));
    super::entrypoint::print_command_error("gate failed");
    let log = run_dir.join("malvin_error.log");
    assert!(log.is_file());
    let body = std::fs::read_to_string(&log).expect("read");
    assert!(body.contains("gate failed"));
    crate::cli::error_run_log::clear_command_error_run_dir();
}

#[test]
fn smoke_prepare_do_prompt_store_loads_defaults() {
    assert!(crate::do_flow::prepare_do_prompt_store().is_ok());
}

#[test]
fn smoke_shared_opts_tee_startup_stdout() {
    let shared = super::SharedOpts {
        model: "m".into(),
        no_force: false,
        no_tee: false,
        no_markdown: false,
        verbose: false,
        doc: false,
        sandbox: false,
    };
    assert!(shared.tee_startup_stdout());
}

#[test]
fn smoke_compose_tidy_prompt_includes_plan_path() {
    use std::collections::HashMap;
    let store = crate::prompts::PromptStore::default_store();
    let mut ctx = HashMap::new();
    ctx.insert(
        "quality_gates_log".to_string(),
        "./_malvin/run/quality_gates.log".to_string(),
    );
    ctx.insert("quality_gates".to_string(), "- `kiss check`\n".to_string());
    ctx.insert("plan_path".to_string(), "./plan.md".to_string());
    let out = super::tidy_flow::compose_tidy_prompt(&store, &ctx).expect("compose");
    assert!(out.contains("./plan.md"));
}
