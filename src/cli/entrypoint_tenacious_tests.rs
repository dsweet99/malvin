use crate::cli::args::Cli;
use clap::{CommandFactory, Parser};
use malvin::config::malvin_config_file::DEFAULT_MAX_LOOPS;

#[test]
fn outer_loop_budget_is_fixed_at_one() {
    assert_eq!(DEFAULT_MAX_LOOPS, 1);
}

#[test]
fn cli_rejects_max_loops_flag() {
    let err = Cli::try_parse_from(["malvin", "--max-loops", "2", "route this"]).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected argument") && msg.contains("--max-loops"),
        "{msg}"
    );
}

#[test]
fn help_omits_max_loops_flag() {
    let help = Cli::command()
        .try_get_matches_from(["malvin", "--help"])
        .expect_err("help")
        .to_string();
    assert!(
        !help.contains("--max-loops"),
        "help must not offer --max-loops:\n{help}"
    );
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
