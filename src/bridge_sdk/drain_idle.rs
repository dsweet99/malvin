//! Per-next-event drain idle with optional child-health extend.

use std::collections::HashSet;
use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

use crate::acp::AgentError;
use crate::sdk_drain_timeout::{sdk_drain_idle_max_wait, sdk_drain_idle_slice};

#[path = "drain_idle_health.rs"]
pub(crate) mod drain_idle_health;
#[path = "drain_idle_turn.rs"]
mod drain_idle_turn;
pub(crate) use drain_idle_turn::DrainIdleTurn;
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
            "{} waiting for {} after {idle:?} of silence",
            self.prefix, self.waiting_for
        ))
    }
}

/// Session fields needed to sample sandbox health during drain idle.
#[derive(Debug, Clone, Copy)]
pub struct DrainIdleHealthCtx<'a> {
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: &'a HashSet<u32>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DrainIdleClock {
    wait_start: Instant,
    idle: Duration,
    idle_deadline: Instant,
}

impl DrainIdleClock {
    pub(crate) fn new(idle: Duration) -> Self {
        let wait_start = Instant::now();
        Self {
            wait_start,
            idle,
            idle_deadline: wait_start + idle,
        }
    }

    pub(crate) fn max_deadline(&self) -> Instant {
        self.wait_start + sdk_drain_idle_max_wait(self.idle)
    }

    pub(crate) fn reset_idle_window(&mut self) {
        let now = Instant::now();
        self.idle_deadline = (now + self.idle).min(self.max_deadline());
    }

    fn remaining_to_max(&self) -> Option<Duration> {
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
#[allow(dead_code)] // used from `#[cfg(test)]` drain-idle modules
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
    let result = await_next_with_idle_using(labels, read, move |slice| async move {
        match health {
            Some(ctx) => sample_drain_health(ctx, slice).await,
            None => DrainHealthVerdict::AppearsHung,
        }
    }, &mut turn.clock)
    .await?;
    turn.clock.reset_idle_window();
    Ok(result)
}

/// Testable core with an injectable health sampler.
pub(crate) async fn await_next_with_idle_using<T, Fut, H, HFut>(
    labels: DrainIdleLabels<'_>,
    read: Fut,
    mut health_sampler: H,
    clock: &mut DrainIdleClock,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
    H: FnMut(Duration) -> HFut,
    HFut: Future<Output = DrainHealthVerdict>,
{
    let idle = clock.idle;
    tokio::pin!(read);
    loop {
        let Some(slice) = clock.slice_duration() else {
            return Err(labels.silence_error(idle));
        };
        if let Ok(result) = tokio::time::timeout(slice, read.as_mut()).await {
            return result;
        }
        let Some(remaining) = clock.remaining_to_max() else {
            return Err(labels.silence_error(idle));
        };
        let verdict = tokio::select! {
            result = read.as_mut() => return result,
            verdict = health_sampler(slice) => verdict,
            () = tokio::time::sleep(remaining) => {
                return Err(labels.silence_error(idle));
            }
        };
        if clock.apply_verdict(verdict).is_err() {
            return Err(labels.silence_error(idle));
        }
    }
}
