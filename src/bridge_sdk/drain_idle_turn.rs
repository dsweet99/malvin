use std::time::Duration;
use tokio::time::Instant;

use crate::acp::AgentError;
use crate::sdk_drain_timeout::sdk_drain_idle_timeout_from_env;

use super::{DrainIdleClock, DrainIdleLabels};

/// Shared wall-clock budget for a multi-event drain (turn / handshake).
pub(crate) struct DrainIdleTurn {
    pub(crate) clock: DrainIdleClock,
    idle: Duration,
}

impl DrainIdleTurn {
    pub(crate) fn new() -> Self {
        let idle = sdk_drain_idle_timeout_from_env();
        Self {
            clock: DrainIdleClock::new(idle),
            idle,
        }
    }

    pub(crate) const fn idle(&self) -> Duration {
        self.idle
    }

    pub(crate) fn check_max_deadline(&self, labels: DrainIdleLabels<'_>) -> Result<(), AgentError> {
        let now = Instant::now();
        if now >= self.clock.max_deadline() {
            Err(labels.turn_budget_error(self.clock.turn_elapsed(), self.clock.turn_limit()))
        } else {
            Ok(())
        }
    }
}
