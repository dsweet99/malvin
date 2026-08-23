//! Per-next-event drain idle with optional child-health extend.

use std::collections::HashSet;
use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

use crate::acp::AgentError;
use crate::child_health::{
    SilenceHealthOutcome, evaluate_after_acp_silence, silence_grace_for_rpc_timeout,
};
use crate::sdk_drain_timeout::{
    sdk_drain_idle_max_wait, sdk_drain_idle_slice, sdk_drain_idle_timeout_from_env,
};

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

    pub(crate) fn max_deadline(self) -> Instant {
        self.wait_start + sdk_drain_idle_max_wait(self.idle)
    }

    fn remaining_to_max(self) -> Option<Duration> {
        let remaining = self
            .max_deadline()
            .saturating_duration_since(Instant::now());
        (!remaining.is_zero()).then_some(remaining)
    }

    pub(crate) fn slice_duration(self) -> Option<Duration> {
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
pub async fn await_next_with_idle<T, Fut>(
    labels: DrainIdleLabels<'_>,
    health: Option<DrainIdleHealthCtx<'_>>,
    read: Fut,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
{
    await_next_with_idle_using(labels, read, move |slice| async move {
        match health {
            Some(ctx) => sample_drain_health(ctx, slice).await,
            None => DrainHealthVerdict::AppearsHung,
        }
    })
    .await
}

/// Testable core with an injectable health sampler.
pub(crate) async fn await_next_with_idle_using<T, Fut, H, HFut>(
    labels: DrainIdleLabels<'_>,
    read: Fut,
    mut health_sampler: H,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
    H: FnMut(Duration) -> HFut,
    HFut: Future<Output = DrainHealthVerdict>,
{
    let idle = sdk_drain_idle_timeout_from_env();
    tokio::pin!(read);
    let mut clock = DrainIdleClock::new(idle);
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

pub(crate) async fn sample_drain_health(
    ctx: DrainIdleHealthCtx<'_>,
    slice: Duration,
) -> DrainHealthVerdict {
    let pids = drain_sample_pids(ctx.process_group_id, ctx.spawn_pid_baseline).await;
    if pids.is_empty() {
        return DrainHealthVerdict::DeadOrZombie;
    }
    aggregate_pid_health(&pids, silence_grace_for_rpc_timeout(slice)).await
}

pub(crate) async fn drain_sample_pids(
    pgid: Option<u32>,
    spawn_pid_baseline: &HashSet<u32>,
) -> Vec<u32> {
    let baseline = spawn_pid_baseline.clone();
    tokio::task::spawn_blocking(move || drain_sample_pids_blocking(pgid, &baseline))
        .await
        .unwrap_or_else(|_| pgid.map_or_else(Vec::new, |id| vec![id]))
}

fn drain_sample_pids_blocking(
    pgid: Option<u32>,
    spawn_pid_baseline: &HashSet<u32>,
) -> Vec<u32> {
    #[cfg(unix)]
    {
        let mut pids: Vec<u32> =
            crate::acp::sandbox_monitor_pids(pgid, spawn_pid_baseline)
                .into_iter()
                .collect();
        if pids.is_empty() && let Some(id) = pgid {
            pids.push(id);
        }
        pids.sort_unstable();
        pids.dedup();
        pids
    }
    #[cfg(not(unix))]
    {
        let _ = spawn_pid_baseline;
        pgid.map_or_else(Vec::new, |id| vec![id])
    }
}

pub(crate) async fn aggregate_pid_health(pids: &[u32], grace: Duration) -> DrainHealthVerdict {
    let mut outcomes = Vec::with_capacity(pids.len());
    for &pid in pids {
        outcomes.push(evaluate_after_acp_silence(pid, grace).await);
    }
    aggregate_health_outcomes(&outcomes)
}

pub(crate) fn aggregate_health_outcomes(outcomes: &[SilenceHealthOutcome]) -> DrainHealthVerdict {
    let mut saw_busy = false;
    let mut saw_alive = false;
    for outcome in outcomes {
        match outcome {
            SilenceHealthOutcome::StillBusyExtendWait => saw_busy = true,
            SilenceHealthOutcome::AppearsHung => saw_alive = true,
            SilenceHealthOutcome::ChildNotRunning | SilenceHealthOutcome::ChildZombie => {}
        }
    }
    if saw_busy {
        DrainHealthVerdict::StillBusy
    } else if saw_alive {
        DrainHealthVerdict::AppearsHung
    } else {
        DrainHealthVerdict::DeadOrZombie
    }
}
