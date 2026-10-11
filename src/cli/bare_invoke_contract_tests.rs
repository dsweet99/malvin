use clap::CommandFactory;

use super::{Cli, parse_cli_with_config_defaults};
use malvin::test_support::test_utils::with_isolated_home;

fn parse(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work| {
        out = Some(parse_cli_with_config_defaults(argv).expect("parse").0);
    });
    out.expect("parsed under isolated home")
}

fn help_has_token(help: &str, flag: &str) -> bool {
    help.split(|c: char| c.is_whitespace() || matches!(c, ',' | '=' | '[' | ']'))
        .any(|tok| tok == flag)
}

fn help_lists_subcommand_line(help: &str, name: &str) -> bool {
    help.lines()
        .any(|line| line.starts_with(&format!("  {name} ")))
}

#[test]
fn removed_do_flag_is_rejected() {
    let err = parse_cli_with_config_defaults(["malvin", "--do", "task"]).expect_err("removed");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("--do"),
        "{msg}"
    );
}

#[test]
fn do_subcommand_is_removed() {
    // Former `do` subcommand is gone; `do` / `task` are ordinary bare requests.
    let cli = parse(&["malvin", "do", "task"]);
    assert!(cli.command.is_none());
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
fn gates_only_route_parses_without_request_and_bare_request_without_subcommand_parses_as_default_route()
 {
    {
        let cli = parse(&["malvin", "-g"]);
        assert!(cli.command.is_none());
        assert!(!cli.has_request());
        assert!(cli.router.gates);
    }
    {
        let cli = parse(&["malvin", "investigate"]);
        assert!(cli.command.is_none());
        assert_eq!(cli.first_request().map(String::as_str), Some("investigate"));
    }
}

#[test]
fn cli_help_omits_removed_subcommands() {
    let mut cmd = Cli::command();
    let help = cmd.render_help().to_string();
    assert!(!help_lists_subcommand_line(&help, "code"));
    assert!(!help_lists_subcommand_line(&help, "do"));
    assert!(!help_lists_subcommand_line(&help, "delight"));
    assert!(!help_has_token(&help, "--do"), "{help}");
    assert!(!help_has_token(&help, "--watch"), "{help}");
    assert!(!help_has_token(&help, "--verbose"), "{help}");
    assert!(!help_has_token(&help, "-v"), "{help}");
    assert!(!help.contains("Bare malvin"), "{help}");
    assert!(!help.contains("bare malvin"), "{help}");
    assert!(!help_lists_subcommand_line(&help, "router"));
    assert!(!help.contains("@code"));
}

#[test]
fn multiple_bare_requests_and_double_dash_requests_parse() {
    let cli = parse(&["malvin", "plan_1.md", "plan_2.md"]);
    assert!(cli.command.is_none());
    assert_eq!(
        cli.requests,
        vec!["plan_1.md".to_string(), "plan_2.md".to_string()]
    );
    let flag_request = parse(&["malvin", "--", "-n"]);
    assert_eq!(flag_request.requests, vec!["-n".to_string()]);
    assert_eq!(flag_request.tagged_requests.len(), 1);
    assert_eq!(flag_request.tagged_requests[0].text, "-n");
    assert!(flag_request.tagged_requests[0].is_router());
    let literal_do = parse(&["malvin", "--", "--do", "task"]);
    assert_eq!(
        literal_do.requests,
        vec!["--do".to_string(), "task".to_string()]
    );
    assert!(
        literal_do
            .tagged_requests
            .iter()
            .all(super::request_argv::TaggedRequest::is_router)
    );
    assert_eq!(
        literal_do
            .tagged_requests
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>(),
        vec!["--do", "task"]
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
