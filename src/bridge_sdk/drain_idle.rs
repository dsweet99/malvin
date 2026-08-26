//! Per-next-event drain idle with optional child-health extend.

use std::collections::HashSet;
use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

use crate::acp::AgentError;
use crate::sdk_drain_timeout::{sdk_drain_idle_max_turn, sdk_drain_idle_max_wait, sdk_drain_idle_slice};

#[path = "drain_idle_health.rs"]
pub(crate) mod drain_idle_health;
#[path = "drain_idle_turn.rs"]
mod drain_idle_turn;
#[path = "drain_idle_wait.rs"]
mod drain_idle_wait;
pub(crate) use drain_idle_turn::DrainIdleTurn;
pub(crate) use drain_idle_wait::{DrainIdleWaitOpts, await_next_with_idle_using};
use drain_idle_health::sample_drain_health;

/// Aggregate sandbox health for one drain-idle slice miss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainHealthVerdict {
    StillBusy,
    AppearsHung,
    DeadOrZombie,
}

/// Labels for silence errors (`DRAIN_IDLE_PREFIX_*` in `src/acp/agent_helpers.rs`).
#[derive(Debug, Clone, Copy)]
pub struct DrainIdleLabels<'a> {
    pub prefix: &'a str,
    pub waiting_for: &'a str,
}

impl DrainIdleLabels<'_> {
    pub(crate) fn silence_error(self, idle: Duration) -> AgentError {
        AgentError(format!(
            "{} waiting for {} after {idle:?} without a bridge event",
            self.prefix, self.waiting_for
        ))
    }

    pub(crate) fn turn_budget_error(self, elapsed: Duration, limit: Duration) -> AgentError {
        AgentError(format!(
            "{} waiting for {} after turn ran {elapsed:?} (limit {limit:?})",
            self.prefix, self.waiting_for
        ))
    }
}

/// Session fields needed to sample sandbox health during drain idle.
#[derive(Debug, Clone, Copy)]
pub struct DrainIdleHealthCtx<'a> {
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: &'a HashSet<u32>,
    /// When true, I/O-bound sandbox work may extend the turn budget like `StillBusy`.
    pub tools_in_flight: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DrainIdleClock {
    wait_start: Instant,
    idle: Duration,
    idle_deadline: Instant,
    turn_deadline: Instant,
}

impl DrainIdleClock {
    pub(crate) fn new(idle: Duration) -> Self {
        let wait_start = Instant::now();
        Self {
            wait_start,
            idle,
            idle_deadline: wait_start + idle,
            turn_deadline: wait_start + sdk_drain_idle_max_wait(idle),
        }
    }

    pub(crate) const fn max_deadline(&self) -> Instant {
        self.turn_deadline
    }

    pub(crate) fn turn_elapsed(&self) -> Duration {
        Instant::now().saturating_duration_since(self.wait_start)
    }

    pub(crate) const fn turn_limit(&self) -> Duration {
        sdk_drain_idle_max_turn(self.idle)
    }

    pub(crate) const fn idle(&self) -> Duration {
        self.idle
    }

    /// Infra-layer turn heartbeat: extend the cumulative turn cap on productive signals.
    pub(crate) fn extend_turn_budget(&mut self, extra: Duration) {
        let cap = self.wait_start + sdk_drain_idle_max_turn(self.idle);
        self.turn_deadline = (self.turn_deadline + extra).min(cap);
        let now = Instant::now();
        self.idle_deadline = (now + self.idle).min(self.turn_deadline);
    }

    pub(crate) fn reset_idle_window(&mut self) {
        let now = Instant::now();
        self.idle_deadline = (now + self.idle).min(self.max_deadline());
    }

    pub(crate) fn remaining_to_max(&self) -> Option<Duration> {
        let remaining = self
            .max_deadline()
            .saturating_duration_since(Instant::now());
        (!remaining.is_zero()).then_some(remaining)
    }

    pub(crate) fn slice_duration(&self) -> Option<Duration> {
        let now = Instant::now();
        if now >= self.idle_deadline || now >= self.max_deadline() {
            return None;
        }
        let cap = self
            .idle_deadline
            .saturating_duration_since(now)
            .min(self.max_deadline().saturating_duration_since(now));
        if cap.is_zero() {
            None
        } else {
            Some(sdk_drain_idle_slice(cap))
        }
    }

    pub(crate) fn apply_verdict(&mut self, verdict: DrainHealthVerdict) -> Result<(), ()> {
        let now = Instant::now();
        if now >= self.max_deadline() {
            return Err(());
        }
        match verdict {
            DrainHealthVerdict::DeadOrZombie => Err(()),
            DrainHealthVerdict::StillBusy => {
                self.idle_deadline = (now + self.idle).min(self.max_deadline());
                Ok(())
            }
            DrainHealthVerdict::AppearsHung => {
                if now >= self.idle_deadline {
                    Err(())
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Await the next bridge/pi read with sliced idle + optional health extend.
#[allow(dead_code)]
pub async fn await_next_with_idle<T, Fut>(
    labels: DrainIdleLabels<'_>,
    health: Option<DrainIdleHealthCtx<'_>>,
    read: Fut,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
{
    let mut turn = DrainIdleTurn::new();
    await_next_with_idle_in_turn(labels, health, read, &mut turn).await
}

/// Await one read within an existing turn budget (cumulative `max_wait` across events).
pub(crate) async fn await_next_with_idle_in_turn<T, Fut>(
    labels: DrainIdleLabels<'_>,
    health: Option<DrainIdleHealthCtx<'_>>,
    read: Fut,
    turn: &mut DrainIdleTurn,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
{
    turn.check_max_deadline(labels)?;
    let tools_in_flight = health.as_ref().is_some_and(|ctx| ctx.tools_in_flight);
    let mut wait = DrainIdleWaitOpts {
        labels,
        clock: &mut turn.clock,
        extend_turn_on_busy_health: tools_in_flight,
    };
    let result = await_next_with_idle_using(&mut wait, read, move |slice| async move {
        let verdict = match health {
            Some(ctx) => sample_drain_health(ctx, slice).await,
            None => DrainHealthVerdict::AppearsHung,
        };
        if tools_in_flight && verdict == DrainHealthVerdict::AppearsHung {
            DrainHealthVerdict::StillBusy
        } else {
            verdict
        }
    })
    .await?;
    turn.clock.reset_idle_window();
    Ok(result)
}
