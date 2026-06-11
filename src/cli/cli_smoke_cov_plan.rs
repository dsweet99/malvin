//! Plan-command CLI smoke tests.

use super::entrypoint::require_kiss_for_cli_command;
use super::{Cli, Commands};

#[test]
fn smoke_cli_parse_plan_subcommand() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "plan", "plan.md"]).expect("parse");
    match cli.command.as_ref() {
        Some(Commands::Plan(p)) => assert_eq!(p.plan_path, "plan.md"),
        _ => panic!("expected Plan"),
    }
}

#[test]
fn smoke_require_kiss_allows_plan_without_kiss_on_path() {
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "plan", "plan.md"]).expect("parse");
    let cmd = cli.command.as_ref().expect("subcommand");
    assert!(require_kiss_for_cli_command(cmd).is_ok());
}

#[test]
fn smoke_prepare_plan_prompt_store_loads_embedded_prompts() {
    assert!(crate::plan_flow::prepare_plan_prompt_store().is_ok());
}

#[test]
fn smoke_shared_opts_tee_startup_stdout() {
    let shared = super::SharedOpts {
        model: "m".into(),
        no_force: false,
        no_tenacious: false,
        no_tee: false,
        no_markdown: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        name: None,
        mini: false,
        mini_max_bash_turns: 32,
    };
    assert!(shared.tee_startup_stdout());
}

#[test]
fn smoke_tidy_kpop_request_includes_constraints() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_kpop_run_artifacts("tidy", Some(tmp.path()))
        .expect("artifacts");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let out = super::tidy_flow::tidy_kpop_request(&store, &artifacts).expect("request");
    assert!(out.contains("Just get quality gates to pass"));
    assert!(out.contains("Satisfy all constraints"));
}
