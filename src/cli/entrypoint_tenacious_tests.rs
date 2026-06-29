use crate::cli::args::{Cli, Commands};
use crate::cli::loop_opts::{
    apply_tenacious, tenacious_budget_guard, TenaciousBudgetGuard, TENACIOUS_MAX_ACP_RETRIES,
    TENACIOUS_MAX_LOOPS,
};
use crate::reliability_tier::{ReliabilityTier, ReliabilityTierFlags};
use clap::{CommandFactory, FromArgMatches};

fn apply_kpop_tenacious(
    kpop: &mut crate::cli::KpopArgs,
    shared: &mut crate::cli::SharedOpts,
    guard: TenaciousBudgetGuard,
) {
    let tier = ReliabilityTier::resolve(ReliabilityTierFlags {
        tenacious: kpop.tenacious,
        no_tenacious: shared.no_tenacious,
    });
    apply_tenacious(
        &mut kpop.max_loops,
        &mut shared.max_acp_retries,
        tier,
        guard,
    );
}

#[test]
fn kpop_defaults_to_tenacious_without_explicit_flag() {
    let matches = Cli::command().get_matches_from(["malvin", "kpop", "investigate"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Kpop(mut kpop)) = cli.command else {
        panic!("expected kpop subcommand");
    };
    assert!(kpop.tenacious);
    let mut shared = cli.shared;
    apply_kpop_tenacious(
        &mut kpop,
        &mut shared,
        tenacious_budget_guard(&matches, "kpop"),
    );
    assert_eq!(kpop.max_loops, TENACIOUS_MAX_LOOPS);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}

#[test]
fn kpop_tenacious_expands_max_loops_and_acp_retries() {
    let matches = Cli::command().get_matches_from(["malvin", "kpop", "--tenacious", "investigate"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Kpop(mut kpop)) = cli.command else {
        panic!("expected kpop subcommand");
    };
    let mut shared = cli.shared;
    apply_kpop_tenacious(
        &mut kpop,
        &mut shared,
        tenacious_budget_guard(&matches, "kpop"),
    );
    assert_eq!(kpop.max_loops, TENACIOUS_MAX_LOOPS);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}

#[test]
fn kpop_no_tenacious_keeps_normal_budgets() {
    let matches =
        Cli::command().get_matches_from(["malvin", "kpop", "--no-tenacious", "investigate"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Kpop(mut kpop)) = cli.command else {
        panic!("expected kpop subcommand");
    };
    assert!(kpop.tenacious);
    assert!(cli.shared.no_tenacious);
    let mut shared = cli.shared;
    apply_kpop_tenacious(
        &mut kpop,
        &mut shared,
        tenacious_budget_guard(&matches, "kpop"),
    );
    assert_eq!(kpop.max_loops, 1);
    assert_eq!(shared.max_acp_retries, crate::config::DEFAULT_MAX_ACP_RETRIES);
}

#[test]
fn kpop_explicit_max_loops_is_not_expanded_by_tenacious_default() {
    let matches = Cli::command().get_matches_from(["malvin", "kpop", "--max-loops", "2", "investigate"]);
    let cli = Cli::from_arg_matches(&matches).expect("parse");
    let Some(Commands::Kpop(mut kpop)) = cli.command else {
        panic!("expected kpop subcommand");
    };
    let mut shared = cli.shared;
    apply_kpop_tenacious(
        &mut kpop,
        &mut shared,
        tenacious_budget_guard(&matches, "kpop"),
    );
    assert_eq!(kpop.max_loops, 2);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}
