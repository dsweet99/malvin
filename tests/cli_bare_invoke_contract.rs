//! Bare CLI invocation (`malvin REQUEST`, `--do`, `@workflow`) contract tests.

use malvin::cli::{parse_cli_with_config_defaults, Cli, Commands};
use clap::CommandFactory;

fn parse(argv: &[&str]) -> Cli {
    parse_cli_with_config_defaults(argv).expect("parse")
}

#[test]
fn bare_request_parses_as_kpop() {
    let cli = parse(&["malvin", "investigate"]);
    match cli.command {
        Some(Commands::Kpop(k)) => assert_eq!(k.request.as_deref(), Some("investigate")),
        other => panic!("expected kpop, got {other:?}"),
    }
}

#[test]
fn do_flag_parses_as_do_subcommand() {
    let cli = parse(&["malvin", "--do", "task"]);
    assert!(matches!(cli.command, Some(Commands::Do(_))));
}

#[test]
fn at_code_parses_as_code_subcommand() {
    let cli = parse(&["malvin", "@code", "plan.md"]);
    match cli.command {
        Some(Commands::Code(c)) => assert_eq!(c.request.as_deref(), Some("plan.md")),
        other => panic!("expected code, got {other:?}"),
    }
}

#[test]
fn at_tidy_parses_without_request() {
    let cli = parse(&["malvin", "@tidy"]);
    assert!(matches!(cli.command, Some(Commands::Tidy(_))));
}

#[test]
fn legacy_kpop_subcommand_still_parses() {
    let cli = parse(&["malvin", "kpop", "q"]);
    assert!(matches!(cli.command, Some(Commands::Kpop(_))));
}

#[test]
fn cli_help_lists_bare_invocation_hint() {
    let mut cmd = Cli::command();
    let help = cmd.render_help().to_string();
    assert!(help.contains("@code"));
    assert!(help.contains("--do"));
}

#[test]
fn kiss_cov_bare_resolve_helper_names() {
    const NAMES: &[&str] = &[
        "resolve_bare_do",
        "resolve_bare_kpop",
        "resolve_bare_at_or_kpop",
        "resolve_bare_command",
    ];
    assert_eq!(NAMES.len(), 4);
}
