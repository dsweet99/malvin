use std::num::NonZeroU32;

use crate::agent_process::AttemptCeiling;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BudgetScopeLayer {
    OuterRouterLoop,
    AcpAttempt,
}

impl BudgetScopeLayer {
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::OuterRouterLoop, Self::AcpAttempt]
    }

    #[must_use]
    pub const fn respects_single_attempt(self) -> bool {
        matches!(self, Self::AcpAttempt)
    }

    #[must_use]
    pub(crate) const fn effective_attempt_ceiling(
        self,
        limit: Option<NonZeroU32>,
        single_attempt: bool,
    ) -> AttemptCeiling {
        if single_attempt && self.respects_single_attempt() {
            return AttemptCeiling::one();
        }
        match limit {
            Some(n) => AttemptCeiling::AtMost(n),
            None => AttemptCeiling::unlimited(),
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
