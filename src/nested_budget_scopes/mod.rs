//! Nested budget scopes (see `concepts.md` §1).
//!
//! Retries and turn limits stack at several independent layers, each with its own knobs
//! and `single_attempt` flags. There is no unified budget tree or coordinator; each layer
//! owns its counter locally. Production references [`BudgetScopeLayer`] at enforcement sites.

/// One independent retry/turn budget layer in concept order (documentation / typing aid; not enforced at runtime).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetScopeLayer {
    /// `OpenRouter` HTTP transport retries per completion (`[agent].max_mini_transport_retries`).
    MiniTransportRetry,
    /// Investigate/WindDown HTTP turns per coder prompt (`--mini-max-http-turns`).
    MiniHttpTurn,
    /// Bash subprocess executions per prompt (`--mini-max-bash-execs`).
    MiniBashExec,
    /// Whole-loop gate retries after failure (`--mini-max-gate-retries`).
    MiniGateIteration,
    /// Context-recovery shrink passes on overflow (`--mini-max-shrink-passes`).
    MiniShrinkPass,
    /// Outer `KPopEngine` / gate-loop iterations (`--max-loops`).
    OuterKPopEngineLoop,
    /// ACP spawn / coder-prompt retries (`--max-acp-retries`).
    AcpSpawnRetry,
}

impl BudgetScopeLayer {
    /// All layers in stable concept order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::MiniTransportRetry,
            Self::MiniHttpTurn,
            Self::MiniBashExec,
            Self::MiniGateIteration,
            Self::MiniShrinkPass,
            Self::OuterKPopEngineLoop,
            Self::AcpSpawnRetry,
        ]
    }

    /// Primary CLI flag controlling this layer, when one exists.
    #[must_use]
    pub const fn cli_flag(self) -> Option<&'static str> {
        match self {
            Self::MiniTransportRetry => None,
            Self::MiniHttpTurn => Some("mini-max-http-turns"),
            Self::MiniBashExec => Some("mini-max-bash-execs"),
            Self::MiniGateIteration => Some("mini-max-gate-retries"),
            Self::MiniShrinkPass => Some("mini-max-shrink-passes"),
            Self::OuterKPopEngineLoop => Some("max-loops"),
            Self::AcpSpawnRetry => Some("max-acp-retries"),
        }
    }

    /// Whether `single_attempt: true` forces one try at this layer.
    #[must_use]
    pub const fn respects_single_attempt(self) -> bool {
        matches!(
            self,
            Self::MiniTransportRetry | Self::MiniGateIteration | Self::AcpSpawnRetry
        )
    }

    /// Whether `OpenRouter` billing/payment failures (402/403) fail immediately without retry.
    #[must_use]
    pub const fn billing_fails_immediately(self) -> bool {
        matches!(self, Self::MiniTransportRetry | Self::MiniGateIteration)
    }

    /// Effective attempt budget for this layer given CLI/config limit and `single_attempt`.
    #[must_use]
    pub fn effective_max_attempts(self, limit: u32, single_attempt: bool) -> u32 {
        if single_attempt && self.respects_single_attempt() {
            1
        } else {
            match self {
                Self::MiniTransportRetry | Self::MiniGateIteration | Self::AcpSpawnRetry => {
                    limit.max(1)
                }
                Self::MiniHttpTurn
                | Self::MiniBashExec
                | Self::MiniShrinkPass
                | Self::OuterKPopEngineLoop => limit,
            }
        }
    }

    /// Effective outer gate-loop iteration budget (at least one when limit is zero).
    #[must_use]
    pub fn effective_outer_loop_iterations(limit: usize) -> usize {
        limit.max(1)
    }
}

#[cfg(test)]
#[path = "nested_budget_scopes_tests.rs"]
mod nested_budget_scopes_tests;
