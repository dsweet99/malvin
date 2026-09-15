use crate::cli::args::Cli;
use crate::cli::loop_opts::TENACIOUS_MAX_LOOPS;
use clap::{CommandFactory, FromArgMatches};

#[test]
fn gates_only_defaults_to_tenacious_without_explicit_flag() {
    let matches = Cli::command().get_matches_from(["malvin", "-g"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    assert!(cli.command.is_none());
    assert!(cli.router.gates);
    let mut shared = cli.shared;
    let mut max_loops = cli.router.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        &matches,
    );
    assert_eq!(max_loops, TENACIOUS_MAX_LOOPS);
    assert_eq!(
        shared.max_acp_retries,
        malvin::config::DEFAULT_MAX_ACP_RETRIES
    );
}

#[test]
fn gates_only_explicit_max_loops_is_not_expanded_by_tenacious_default() {
    let matches = Cli::command().get_matches_from(["malvin", "-g", "--max-loops", "2"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let mut shared = cli.shared;
    let mut max_loops = cli.router.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        &matches,
    );
    assert_eq!(max_loops, 2);
    assert_eq!(
        shared.max_acp_retries,
        malvin::config::DEFAULT_MAX_ACP_RETRIES
    );
}

#[test]
fn default_route_tenacious_expands_max_loops_and_leaves_acp_retries() {
    let matches = Cli::command().get_matches_from(["malvin", "route this"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    assert!(cli.command.is_none());
    let mut shared = cli.shared;
    let mut max_loops = cli.router.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        &matches,
    );
    assert_eq!(
        shared.max_acp_retries,
        malvin::config::DEFAULT_MAX_ACP_RETRIES
    );
    assert_eq!(max_loops, TENACIOUS_MAX_LOOPS);
}

#[test]
fn default_route_explicit_max_acp_retries_is_not_expanded_by_tenacious_default() {
    let matches =
        Cli::command().get_matches_from(["malvin", "--max-acp-retries", "4", "route this"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let mut shared = cli.shared;
    let mut max_loops = cli.router.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        &matches,
    );
    assert_eq!(shared.max_acp_retries, 4);
    assert_eq!(max_loops, TENACIOUS_MAX_LOOPS);
}

#[test]
fn default_route_explicit_max_loops_is_not_expanded_by_tenacious_default() {
    let matches = Cli::command().get_matches_from(["malvin", "--max-loops", "2", "route this"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let mut shared = cli.shared;
    let mut max_loops = cli.router.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        &matches,
    );
    assert_eq!(max_loops, 2);
    assert_eq!(
        shared.max_acp_retries,
        malvin::config::DEFAULT_MAX_ACP_RETRIES
    );
}
