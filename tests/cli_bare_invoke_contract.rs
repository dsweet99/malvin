//! CLI subcommand contract tests.

mod common;

use malvin::cli::{parse_cli_with_config_defaults, Cli, Commands};
use clap::CommandFactory;
use common::with_isolated_home;

fn parse(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work, _home| {
        out = Some(
            parse_cli_with_config_defaults(argv)
                .expect("parse")
                .0,
        );
    });
    out.expect("parsed under isolated home")
}

fn help_lists_subcommand_line(help: &str, name: &str) -> bool {
    help.lines()
        .any(|line| line.starts_with(&format!("  {name} ")))
}

#[test]
fn do_subcommand_parses() {
    let cli = parse(&["malvin", "do", "task"]);
    match cli.command {
        Some(Commands::Do(d)) => assert_eq!(d.request.as_deref(), Some("task")),
        other => panic!("expected do, got {other:?}"),
    }
}

#[test]
fn code_subcommand_still_parses() {
    let cli = parse(&["malvin", "code", "plan.md"]);
    match cli.command {
        Some(Commands::Code(c)) => assert_eq!(c.requests.as_slice(), &["plan.md"]),
        other => panic!("expected code, got {other:?}"),
    }
}

#[test]
fn tidy_subcommand_still_parses() {
    let cli = parse(&["malvin", "tidy"]);
    assert!(matches!(cli.command, Some(Commands::Tidy(_))));
}

#[test]
fn kpop_subcommand_is_removed() {
    let err = parse_cli_with_config_defaults(["malvin", "kpop", "investigate"]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unrecognized subcommand") || msg.contains("unexpected"),
        "expected unknown-subcommand error, got: {msg}"
    );
}

#[test]
fn bare_request_without_subcommand_parses_as_default_route() {
    let cli = parse(&["malvin", "investigate"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("investigate"));
}

#[test]
fn cli_help_does_not_list_kpop_subcommand() {
    let mut cmd = Cli::command();
    let help = cmd.render_help().to_string();
    assert!(!help_lists_subcommand_line(&help, "code"));
    assert!(!help_lists_subcommand_line(&help, "kpop"));
    assert!(help.contains("do"));
    assert!(!help.contains("router"));
    assert!(!help.contains("@code"));
}

#[test]
fn code_subcommand_accepts_multiple_plans() {
    let cli = parse(&["malvin", "code", "plan_1.md", "plan_2.md"]);
    match cli.command {
        Some(Commands::Code(c)) => {
            assert_eq!(c.requests.as_slice(), &["plan_1.md", "plan_2.md"]);
        }
        other => panic!("expected code, got {other:?}"),
    }
}
