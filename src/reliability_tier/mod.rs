//! Tenacious resilience tier (see `concepts.md` §7).
//!
//! `--no-tenacious` opts into conservative gate-loop and ACP retry budgets; default CLI
//! parsing keeps tenacious expansion on. Production resolves tiers via
//! [`ReliabilityTier::resolve`] in `apply_gate_loop_tenacious` / `apply_tenacious`.

/// Cross-cutting reliability tier for outer gate-loop and ACP spawn retry budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReliabilityTier {
    /// Expanded `--max-loops` and `--max-acp-retries` unless explicitly set on the CLI.
    Tenacious,
    /// CLI default budgets without tenacious expansion.
    Conservative,
}

/// CLI flags that select a [`ReliabilityTier`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReliabilityTierFlags {
    pub tenacious: bool,
    pub no_tenacious: bool,
}

impl ReliabilityTier {
    /// Resolve tier from per-subcommand `--tenacious` and global `--no-tenacious`.
    #[must_use]
    pub const fn resolve(flags: ReliabilityTierFlags) -> Self {
        if flags.tenacious && !flags.no_tenacious {
            Self::Tenacious
        } else {
            Self::Conservative
        }
    }
}

#[cfg(test)]
#[path = "reliability_tier_tests.rs"]
mod reliability_tier_tests;
