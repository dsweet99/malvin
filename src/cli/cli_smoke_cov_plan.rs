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
