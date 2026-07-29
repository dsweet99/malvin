use crate::cli::args::{Cli, Commands};
use crate::cli::loop_opts::{
    apply_tenacious, tenacious_budget_guard, TenaciousBudgetGuard, TENACIOUS_MAX_ACP_RETRIES,
    TENACIOUS_MAX_LOOPS,
};
use crate::cli::tidy_flow::TidyArgs;
use crate::reliability_tier::{ReliabilityTier, ReliabilityTierFlags};
use clap::{CommandFactory, FromArgMatches};

fn apply_tidy_tenacious(
    tidy: &mut TidyArgs,
    shared: &mut crate::cli::SharedOpts,
    guard: TenaciousBudgetGuard,
) {
    let tier = ReliabilityTier::resolve(ReliabilityTierFlags {
        tenacious: tidy.tenacious,
        no_tenacious: shared.no_tenacious,
    });
    apply_tenacious(
        &mut tidy.max_loops,
        &mut shared.max_acp_retries,
        tier,
        guard,
    );
}

#[test]
fn tidy_defaults_to_tenacious_without_explicit_flag() {
    let matches = Cli::command().get_matches_from(["malvin", "tidy"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Tidy(mut tidy)) = cli.command else {
        panic!("expected tidy subcommand");
    };
    assert!(tidy.tenacious);
    let mut shared = cli.shared;
    apply_tidy_tenacious(
        &mut tidy,
        &mut shared,
        tenacious_budget_guard(&matches, "tidy"),
    );
    assert_eq!(tidy.max_loops, TENACIOUS_MAX_LOOPS);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}

#[test]
fn tidy_tenacious_expands_max_loops_and_acp_retries() {
    let matches = Cli::command().get_matches_from(["malvin", "tidy", "--tenacious"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Tidy(mut tidy)) = cli.command else {
        panic!("expected tidy subcommand");
    };
    let mut shared = cli.shared;
    apply_tidy_tenacious(
        &mut tidy,
        &mut shared,
        tenacious_budget_guard(&matches, "tidy"),
    );
    assert_eq!(tidy.max_loops, TENACIOUS_MAX_LOOPS);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}

#[test]
fn tidy_no_tenacious_keeps_normal_budgets() {
    let matches = Cli::command().get_matches_from(["malvin", "tidy", "--no-tenacious"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Tidy(mut tidy)) = cli.command else {
        panic!("expected tidy subcommand");
    };
    assert!(tidy.tenacious);
    assert!(cli.shared.no_tenacious);
    let mut shared = cli.shared;
    apply_tidy_tenacious(
        &mut tidy,
        &mut shared,
        tenacious_budget_guard(&matches, "tidy"),
    );
    assert_eq!(tidy.max_loops, crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE);
    assert_eq!(shared.max_acp_retries, crate::config::DEFAULT_MAX_ACP_RETRIES);
}

#[test]
fn tidy_explicit_max_loops_is_not_expanded_by_tenacious_default() {
    let matches = Cli::command().get_matches_from(["malvin", "tidy", "--max-loops", "2"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Tidy(mut tidy)) = cli.command else {
        panic!("expected tidy subcommand");
    };
    let mut shared = cli.shared;
    apply_tidy_tenacious(
        &mut tidy,
        &mut shared,
        tenacious_budget_guard(&matches, "tidy"),
    );
    assert_eq!(tidy.max_loops, 2);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}

#[test]
fn default_route_tenacious_expands_max_loops_and_acp_retries() {
    let matches = Cli::command().get_matches_from(["malvin", "route this"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    assert!(cli.command.is_none());
    let mut shared = cli.shared;
    let mut max_loops = cli.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        shared.no_tenacious,
        &matches,
    );
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
    assert_eq!(max_loops, TENACIOUS_MAX_LOOPS);
}

#[test]
fn default_route_no_tenacious_keeps_normal_budgets() {
    let matches = Cli::command().get_matches_from(["malvin", "--no-tenacious", "route this"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let mut shared = cli.shared;
    let mut max_loops = cli.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        shared.no_tenacious,
        &matches,
    );
    assert_eq!(shared.max_acp_retries, crate::config::DEFAULT_MAX_ACP_RETRIES);
    assert_eq!(max_loops, crate::malvin_config_file::DEFAULT_MAX_LOOPS);
}

#[test]
fn default_route_explicit_max_acp_retries_is_not_expanded_by_tenacious_default() {
    let matches =
        Cli::command().get_matches_from(["malvin", "--max-acp-retries", "4", "route this"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let mut shared = cli.shared;
    let mut max_loops = cli.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        shared.no_tenacious,
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
    let mut max_loops = cli.max_loops;
    crate::cli::loop_opts::apply_default_route_tenacious(
        &mut max_loops,
        &mut shared.max_acp_retries,
        shared.no_tenacious,
        &matches,
    );
    assert_eq!(max_loops, 2);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}
