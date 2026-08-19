mod common;

use clap::CommandFactory;
use common::with_isolated_home;
use malvin::cli::{Cli, Commands, parse_cli_with_config_defaults};

fn parse(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work, _home| {
        out = Some(parse_cli_with_config_defaults(argv).expect("parse").0);
    });
    out.expect("parsed under isolated home")
}

fn help_lists_subcommand_line(help: &str, name: &str) -> bool {
    help.lines()
        .any(|line| line.starts_with(&format!("  {name} ")))
}

#[test]
fn do_flag_parses() {
    let cli = parse(&["malvin", "--do", "task"]);
    assert!(cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("task"));
    assert!(cli.command.is_none());
}

#[test]
fn do_subcommand_is_removed() {
    let err = parse_cli_with_config_defaults(["malvin", "do", "task"]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("unrecognized"),
        "expected parse error for removed do subcommand, got: {msg}"
    );
}

#[test]
fn code_is_not_a_subcommand_and_parses_as_bare_request() {
    let cli = parse(&["malvin", "code"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("code"));
}

#[test]
fn tidy_subcommand_still_parses() {
    let cli = parse(&["malvin", "tidy"]);
    assert!(matches!(cli.command, Some(Commands::Tidy(_))));
}

#[test]
fn bare_request_without_subcommand_parses_as_default_route() {
    let cli = parse(&["malvin", "investigate"]);
    assert!(cli.command.is_none());
    assert!(!cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("investigate"));
}

#[test]
fn cli_help_omits_removed_subcommands() {
    let mut cmd = Cli::command();
    let help = cmd.render_help().to_string();
    assert!(!help_lists_subcommand_line(&help, "code"));
    assert!(!help_lists_subcommand_line(&help, "do"));
    assert!(!help_lists_subcommand_line(&help, "delight"));
    assert!(help.contains("--do"));
    assert!(!help_lists_subcommand_line(&help, "router"));
    assert!(!help.contains("@code"));
}

#[test]
fn multiple_bare_request_args_are_rejected() {
    let err = parse_cli_with_config_defaults(["malvin", "plan_1.md", "plan_2.md"]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("too many"),
        "expected parse error for multiple bare requests, got: {msg}"
    );
}
