//! `kpop` subcommand contract tests.

use malvin::cli::{parse_cli_with_config_defaults, Cli, Commands};
use clap::CommandFactory;

fn parse(argv: &[&str]) -> Cli {
    parse_cli_with_config_defaults(argv)
        .expect("parse")
        .0
}

fn help_lists_subcommand_line(help: &str, name: &str) -> bool {
    help.lines()
        .any(|line| line.starts_with(&format!("  {name} ")))
}

#[test]
fn kpop_request_parses() {
    let cli = parse(&["malvin", "kpop", "investigate"]);
    match cli.command {
        Some(Commands::Kpop(k)) => assert_eq!(k.requests.as_slice(), &["investigate"]),
        other => panic!("expected kpop, got {other:?}"),
    }
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
fn kpop_subcommand_parses_multiple_requests() {
    let cli = parse(&["malvin", "kpop", "req_a.md", "req_b.md"]);
    match cli.command {
        Some(Commands::Kpop(k)) => {
            assert_eq!(k.requests.as_slice(), &["req_a.md", "req_b.md"]);
        }
        other => panic!("expected kpop, got {other:?}"),
    }
}

#[test]
fn bare_request_without_subcommand_parses_as_default_route() {
    let cli = parse(&["malvin", "investigate"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("investigate"));
}

#[test]
fn cli_help_lists_kpop_subcommand() {
    let mut cmd = Cli::command();
    let help = cmd.render_help().to_string();
    assert!(!help_lists_subcommand_line(&help, "code"));
    assert!(help_lists_subcommand_line(&help, "kpop"));
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
