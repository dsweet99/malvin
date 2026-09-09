use clap::ArgMatches;

use crate::reliability_tier::{ReliabilityTier, ReliabilityTierFlags};

use super::config_defaults::global_flag_from_command_line;

pub const TENACIOUS_MAX_LOOPS: usize = 9999;
pub const TENACIOUS_MAX_ACP_RETRIES: u32 = 9999;

pub struct TenaciousBudgetGuard {
    pub max_loops_explicit: bool,
    pub max_acp_retries_explicit: bool,
}

pub fn apply_default_route_tenacious(
    max_loops: &mut usize,
    max_acp_retries: &mut u32,
    no_tenacious: bool,
    matches: &ArgMatches,
) {
    let tier = ReliabilityTier::resolve(ReliabilityTierFlags {
        tenacious: true,
        no_tenacious,
    });
    apply_tenacious(
        max_loops,
        max_acp_retries,
        tier,
        TenaciousBudgetGuard {
            max_loops_explicit: global_flag_from_command_line(matches, "max_loops"),
            max_acp_retries_explicit: global_flag_from_command_line(matches, "max_acp_retries"),
        },
    );
}

#[allow(clippy::missing_const_for_fn)]
pub fn apply_tenacious(
    max_loops: &mut usize,
    max_acp_retries: &mut u32,
    tier: ReliabilityTier,
    guard: TenaciousBudgetGuard,
) {
    if tier == ReliabilityTier::Tenacious {
        if !guard.max_loops_explicit {
            *max_loops = TENACIOUS_MAX_LOOPS;
        }
        if !guard.max_acp_retries_explicit {
            *max_acp_retries = TENACIOUS_MAX_ACP_RETRIES;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_tenacious_sets_large_budgets() {
        let mut loops = 1usize;
        let mut retries = 3u32;
        apply_tenacious(
            &mut loops,
            &mut retries,
            ReliabilityTier::Tenacious,
            TenaciousBudgetGuard {
                max_loops_explicit: false,
                max_acp_retries_explicit: false,
            },
        );
        assert_eq!(loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn apply_tenacious_skips_explicit_max_loops() {
        let mut loops = 3usize;
        let mut retries = 3u32;
        apply_tenacious(
            &mut loops,
            &mut retries,
            ReliabilityTier::Tenacious,
            TenaciousBudgetGuard {
                max_loops_explicit: true,
                max_acp_retries_explicit: false,
            },
        );
        assert_eq!(loops, 3);
        assert_eq!(retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn apply_default_route_tenacious_expands_unless_opted_out() {
        use crate::cli::Cli;
        use clap::CommandFactory;
        let matches = Cli::command().get_matches_from(["malvin", "topic"]);
        let mut loops = 1usize;
        let mut retries = 3u32;
        apply_default_route_tenacious(&mut loops, &mut retries, false, &matches);
        assert_eq!(loops, TENACIOUS_MAX_LOOPS);
        assert_eq!(retries, TENACIOUS_MAX_ACP_RETRIES);
    }

    #[test]
    fn apply_tenacious_conservative_leaves_budgets() {
        let mut loops = 3usize;
        let mut retries = 5u32;
        apply_tenacious(
            &mut loops,
            &mut retries,
            ReliabilityTier::Conservative,
            TenaciousBudgetGuard {
                max_loops_explicit: false,
                max_acp_retries_explicit: false,
            },
        );
        assert_eq!(loops, 3);
        assert_eq!(retries, 5);
    }
}
