use crate::cli::args::Cli;
use clap::{CommandFactory, Parser};
use malvin::config::malvin_config_file::DEFAULT_MAX_LOOPS;

#[test]
fn outer_loop_budget_defaults_to_999() {
    assert_eq!(DEFAULT_MAX_LOOPS, 999);
    let cli = Cli::try_parse_from(["malvin", "route this"]).expect("parse");
    assert_eq!(cli.router.max_loops, DEFAULT_MAX_LOOPS);
}

#[test]
fn cli_accepts_max_loops_flag() {
    let cli = Cli::try_parse_from(["malvin", "--max-loops", "2", "route this"]).expect("parse");
    assert_eq!(cli.router.max_loops, 2);
    let eq = Cli::try_parse_from(["malvin", "--max-loops=4", "route this"]).expect("parse");
    assert_eq!(eq.router.max_loops, 4);
    let err = Cli::try_parse_from(["malvin", "--max-loops", "0", "route this"]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("--max-loops") && msg.contains("positive integer"),
        "{msg}"
    );
}

#[test]
fn help_lists_max_loops_flag() {
    let help = Cli::command()
        .try_get_matches_from(["malvin", "--help"])
        .expect_err("help")
        .to_string();
    assert!(
        help.contains("--max-loops"),
        "help must offer --max-loops:\n{help}"
    );
    assert!(help.contains("999"), "help must show default 999:\n{help}");
}

#[test]
fn default_route_leaves_acp_retries_at_the_configured_default() {
    let cli = Cli::try_parse_from(["malvin", "route this"]).expect("parse");
    assert_eq!(
        cli.shared.max_acp_retries,
        malvin::config::DEFAULT_MAX_ACP_RETRIES
    );
    let explicit =
        Cli::try_parse_from(["malvin", "--max-acp-retries", "4", "route this"]).expect("parse");
    assert_eq!(explicit.shared.max_acp_retries, 4);
}
