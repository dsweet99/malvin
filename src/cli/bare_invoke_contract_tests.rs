use clap::CommandFactory;

use super::{Cli, parse_cli_with_config_defaults};
use malvin::test_utils::with_isolated_home;

fn parse(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work| {
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
    assert_eq!(cli.first_request().map(String::as_str), Some("task"));
    assert!(cli.command.is_none());
}

#[test]
fn do_subcommand_is_removed() {
    // Former `do` subcommand is gone; `do` / `task` are ordinary bare requests.
    let cli = parse(&["malvin", "do", "task"]);
    assert!(cli.command.is_none());
    assert!(!cli.do_workflow);
    assert_eq!(cli.requests, vec!["do".to_string(), "task".to_string()]);
}

#[test]
fn init_is_not_a_subcommand_and_parses_as_bare_request() {
    let cli = parse(&["malvin", "init"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.first_request().map(String::as_str), Some("init"));
}

#[test]
fn code_is_not_a_subcommand_and_parses_as_bare_request() {
    let cli = parse(&["malvin", "code"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.first_request().map(String::as_str), Some("code"));
}

#[test]
fn gates_only_route_parses_without_request() {
    let cli = parse(&["malvin", "-g"]);
    assert!(cli.command.is_none());
    assert!(!cli.has_request());
    assert!(cli.router.gates);
}

#[test]
fn bare_request_without_subcommand_parses_as_default_route() {
    let cli = parse(&["malvin", "investigate"]);
    assert!(cli.command.is_none());
    assert!(!cli.do_workflow);
    assert_eq!(cli.first_request().map(String::as_str), Some("investigate"));
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
fn multiple_bare_request_args_parse_as_independent_requests() {
    let cli = parse(&["malvin", "plan_1.md", "plan_2.md"]);
    assert!(cli.command.is_none());
    assert!(!cli.do_workflow);
    assert_eq!(
        cli.requests,
        vec!["plan_1.md".to_string(), "plan_2.md".to_string()]
    );
}

#[test]
fn adaptix_subcommand_is_removed() {
    assert!(
        !Cli::command()
            .get_subcommands()
            .any(|c| c.get_name() == "adaptix" || c.get_name() == "inspire"),
        "inspire/adaptix must not be clap subcommands"
    );
    let cli = parse(&["malvin", "adaptix"]);
    assert!(cli.command.is_none());
    assert_eq!(cli.first_request().map(String::as_str), Some("adaptix"));
}
