use super::entrypoint::try_tokio_runtime;
use super::{Cli, Commands};

fn smoke_has_source_files_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(!crate::source_detect::has_source_files(tmp.path()));
}

fn smoke_has_source_files_detects_rs() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("x.rs"), "").unwrap();
    assert!(crate::source_detect::has_source_files(tmp.path()));
}

fn smoke_merge_acp_and_timing_results() {
    use crate::acp_post_run::merge_acp_and_timing_results;
    assert_eq!(merge_acp_and_timing_results(Ok(()), Ok(())), Ok(()));
    assert_eq!(
        merge_acp_and_timing_results(Err("acp".into()), Err(std::io::Error::other("io"))),
        Err("acp".into())
    );
}

fn smoke_prefer_primary_over_secondary() {
    use crate::acp_post_run::prefer_primary_over_secondary;
    assert_eq!(prefer_primary_over_secondary(Ok(()), Ok(()), "x"), Ok(()));
    assert_eq!(
        prefer_primary_over_secondary(Ok(()), Err("b".into()), "x"),
        Err("b".into())
    );
}

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

fn smoke_agent_io_options_maps_flags() {
    use super::{AgentStdoutTeeFlags, WorkflowCliOptions, agent_io_options};
    let shared = super::SharedOpts {
        background: false,
        model: crate::model_id::parse_model_id("cursor:m").expect("model"),
        no_force: false,
        no_tenacious: false,
        gates: false,

        quiet: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        git: false,
        creative: None,
        no_kpop: false,
    };
    let io = agent_io_options(
        &shared,
        WorkflowCliOptions { force: true },
        AgentStdoutTeeFlags {
            emit_stdout_markdown: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
        },
    );
    assert!(io.force);
    assert!(!io.no_tee);
    assert!(io.raw_output);
    assert!(!io.show_thoughts_on_stdout);
    assert!(io.emit_stdout_markdown);
    assert!(!io.log_full_outgoing_prompts);
}

fn init_is_not_a_subcommand_and_parses_as_bare_request() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "init"]).expect("parse");
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("init"));
}

fn smoke_cli_parse_models_subcommand() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "admin", "models"]).unwrap();
    assert!(matches!(
        cli.command,
        Some(Commands::Admin(crate::cli::AdminArgs {
            command: crate::cli::AdminCommand::Models(_),
        }))
    ));
}

fn smoke_try_tokio_runtime_builds_multi_thread() {
    let _rt = try_tokio_runtime().expect("tokio runtime");
}

fn smoke_tidy_effective_max_loops() {
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(0), 1);
    assert_eq!(super::tidy_flow::effective_tidy_max_loops(3), 3);
}

fn smoke_emit_command_line_writes_log() {
    use std::path::Path;
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    super::run_emit::emit_command_line(Path::new(&run_dir), false).expect("emit");
    assert!(run_dir.join("command.log").is_file());
}

fn smoke_format_logs_dir_under_run_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let logs = crate::format_logs_dir(&run_dir).expect("logs dir");
    assert!(logs.contains("run"));
}

fn smoke_run_emit_echo_primary_noop_when_not_plain() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "plan").expect("write plan");
    super::run_emit::echo_primary_to_stdout(&plan, false).expect("echo");
}

fn smoke_print_command_error_writes_run_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    crate::run_id::activate_run(run_dir.clone());
    super::entrypoint::print_command_error("gate failed");
    let log = run_dir.join("malvin_error.log");
    assert!(log.is_file());
    let body = std::fs::read_to_string(&log).expect("read");
    assert!(body.contains("gate failed"));
    super::error_run_log::clear_command_error_run_dir();
}

fn smoke_prepare_do_prompt_store_loads_defaults() {
    assert!(crate::do_flow::prepare_do_prompt_store().is_ok());
}

fn smoke_prepare_router_prompt_store_loads_defaults() {
    assert!(crate::router_flow::prepare_router_prompt_store().is_ok());
}

#[test]
fn kiss_bundled_cli_cli_smoke_cov() {
    smoke_has_source_files_empty_dir();
    smoke_has_source_files_detects_rs();
    smoke_merge_acp_and_timing_results();
    smoke_prefer_primary_over_secondary();
    smoke_merge_acp_with_workspace_session_restore();
    smoke_merge_acp_with_workspace_session_restore_and_check_abort_no_result_file();
    smoke_agent_io_options_maps_flags();
    init_is_not_a_subcommand_and_parses_as_bare_request();
    smoke_cli_parse_models_subcommand();
    smoke_try_tokio_runtime_builds_multi_thread();
    smoke_tidy_effective_max_loops();
    smoke_emit_command_line_writes_log();
    smoke_format_logs_dir_under_run_dir();
    smoke_run_emit_echo_primary_noop_when_not_plain();
    smoke_print_command_error_writes_run_log();
    smoke_prepare_do_prompt_store_loads_defaults();
    smoke_prepare_router_prompt_store_loads_defaults();
}
