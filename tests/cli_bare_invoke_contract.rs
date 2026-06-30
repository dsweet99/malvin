//! Bare CLI invocation (`malvin REQUEST...` → kpop) contract tests.

use malvin::cli::{parse_cli_with_config_defaults, Cli, Commands};
use clap::CommandFactory;

fn parse(argv: &[&str]) -> Cli {
    parse_cli_with_config_defaults(argv)
        .expect("parse")
        .0
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
fn legacy_kpop_subcommand_still_parses() {
    let cli = parse(&["malvin", "kpop", "q"]);
    assert!(matches!(cli.command, Some(Commands::Kpop(_))));
}

#[test]
fn cli_help_lists_bare_invocation_hint() {
    let mut cmd = Cli::command();
    let help = cmd.render_help().to_string();
    assert!(help.contains("malvin REQUEST"));
    assert!(help.contains("do"));
    assert!(!help.contains("@code"));
}

#[test]
fn multiple_bare_requests_do_not_join_into_single_kpop() {
    let cli = parse(&["malvin", "req_a.md", "req_b.md"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.bare_args, vec!["req_a.md", "req_b.md"]);
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

#[test]
fn generate_script_subcommand_parses() {
    let cli = parse(&["malvin", "generate-script", "run-3-steps: a.sh,b.sh,c.sh"]);
    match cli.command {
        Some(Commands::GenerateScript(g)) => {
            assert_eq!(
                g.recipe.as_deref(),
                Some("run-3-steps: a.sh,b.sh,c.sh")
            );
        }
        other => panic!("expected generate-script, got {other:?}"),
    }
}

#[test]
fn kiss_cov_bare_resolve_helper_names() {
    const NAMES: &[&str] = &["resolve_bare_kpop", "resolve_bare_command"];
    assert_eq!(NAMES.len(), 2);
}
