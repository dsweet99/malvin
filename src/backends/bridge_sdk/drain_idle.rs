use std::collections::HashSet;
use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

use crate::agent_process::AgentError;
use crate::sdk_drain_timeout::sdk_drain_idle_slice;

#[path = "drain_idle_health.rs"]
pub(crate) mod drain_idle_health;
#[path = "drain_idle_turn.rs"]
mod drain_idle_turn;
#[path = "drain_idle_wait.rs"]
mod drain_idle_wait;
use drain_idle_health::sample_drain_health;
pub(crate) use drain_idle_turn::DrainIdleTurn;
pub(crate) use drain_idle_wait::{DrainIdleWaitOpts, await_next_with_idle_using};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainHealthVerdict {
    StillBusy,
    AppearsHung,
    DeadOrZombie,
}

#[derive(Debug, Clone, Copy)]
pub struct DrainIdleLabels<'a> {
    pub prefix: &'a str,
    pub waiting_for: &'a str,
}

impl DrainIdleLabels<'_> {
    pub(crate) fn silence_error_detail(self, idle: Duration, tools_in_flight: bool) -> AgentError {
        let why = if tools_in_flight {
            "bridge quiet while tools_in_flight"
        } else {
            "bridge quiet; likely hung or stalled"
        };
        AgentError::session_dead(format!(
            "{} waiting for {} after {idle:?} without a bridge event ({why})",
            self.prefix, self.waiting_for
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrainIdleHealthCtx<'a> {
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: &'a HashSet<u32>,
    pub tools_in_flight: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DrainIdleClock {
    idle: Duration,
    idle_deadline: Instant,
}

impl DrainIdleClock {
    pub(crate) fn new(idle: Duration) -> Self {
        Self {
            idle,
            idle_deadline: Instant::now() + idle,
        }
    }

    pub(crate) const fn idle(&self) -> Duration {
        self.idle
    }

    pub(crate) fn reset_idle_window(&mut self) {
        self.idle_deadline = Instant::now() + self.idle;
    }

    pub(crate) fn slice_duration(&self) -> Option<Duration> {
        let remaining = self.idle_deadline.saturating_duration_since(Instant::now());
        (!remaining.is_zero()).then(|| sdk_drain_idle_slice(remaining))
    }

    pub(crate) fn apply_verdict(&mut self, verdict: DrainHealthVerdict) -> Result<(), ()> {
        let now = Instant::now();
        match verdict {
            DrainHealthVerdict::DeadOrZombie => Err(()),
            DrainHealthVerdict::StillBusy => {
                self.idle_deadline = now + self.idle;
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

/// Test helper: one-shot idle await with a fresh turn budget.
#[cfg(test)]
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

pub(crate) async fn await_next_with_idle_in_turn<T, Fut>(
    labels: DrainIdleLabels<'_>,
    health: Option<DrainIdleHealthCtx<'_>>,
    read: Fut,
    turn: &mut DrainIdleTurn,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
{
    let tools_in_flight = health.as_ref().is_some_and(|ctx| ctx.tools_in_flight);
    let mut wait = DrainIdleWaitOpts {
        labels,
        clock: &mut turn.clock,
        tools_in_flight,
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
