
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetScopeLayer {
    OuterKPopEngineLoop,
    AcpSpawnRetry,
}

impl BudgetScopeLayer {
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::OuterKPopEngineLoop, Self::AcpSpawnRetry]
    }

    #[must_use]
    pub const fn respects_single_attempt(self) -> bool {
        matches!(self, Self::AcpSpawnRetry)
    }

    #[must_use]
    pub fn effective_max_attempts(self, limit: u32, single_attempt: bool) -> u32 {
        if single_attempt && self.respects_single_attempt() {
            1
        } else {
            match self {
                Self::AcpSpawnRetry => limit.max(1),
                Self::OuterKPopEngineLoop => limit,
            }
        }
    }

    #[must_use]
    pub fn effective_outer_loop_iterations(limit: usize) -> usize {
        limit.max(1)
    }
}

#[cfg(test)]
#[path = "nested_budget_scopes_tests.rs"]
mod nested_budget_scopes_tests;
