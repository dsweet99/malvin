use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

use crate::acp::AgentError;

use super::{DrainHealthVerdict, DrainIdleClock, DrainIdleLabels};

pub(crate) struct DrainIdleWaitOpts<'a> {
    pub labels: DrainIdleLabels<'a>,
    pub clock: &'a mut DrainIdleClock,
    pub extend_turn_on_busy_health: bool,
}

impl DrainIdleWaitOpts<'_> {
    fn turn_budget_error(&self) -> AgentError {
        self.labels
            .turn_budget_error(self.clock.turn_elapsed(), self.clock.turn_limit())
    }

    fn slice_miss_error(&self, idle: Duration) -> AgentError {
        if Instant::now() >= self.clock.max_deadline() {
            self.turn_budget_error()
        } else {
            self.labels.silence_error(idle)
        }
    }

    fn verdict_miss_error(&self, idle: Duration) -> AgentError {
        if Instant::now() >= self.clock.max_deadline() {
            self.turn_budget_error()
        } else {
            self.labels.silence_error(idle)
        }
    }
}

/// Testable core with an injectable health sampler.
pub(crate) async fn await_next_with_idle_using<T, Fut, H, HFut>(
    opts: &mut DrainIdleWaitOpts<'_>,
    read: Fut,
    mut health_sampler: H,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
    H: FnMut(Duration) -> HFut,
    HFut: Future<Output = DrainHealthVerdict>,
{
    let idle = opts.clock.idle();
    tokio::pin!(read);
    loop {
        let Some(slice) = opts.clock.slice_duration() else {
            return Err(opts.slice_miss_error(idle));
        };
        if let Ok(result) = tokio::time::timeout(slice, read.as_mut()).await {
            return result;
        }
        let Some(remaining) = opts.clock.remaining_to_max() else {
            return Err(opts.turn_budget_error());
        };
        let verdict = tokio::select! {
            result = read.as_mut() => return result,
            verdict = health_sampler(slice) => verdict,
            () = tokio::time::sleep(remaining) => {
                return Err(opts.turn_budget_error());
            }
        };
        if opts.clock.apply_verdict(verdict).is_err() {
            return Err(opts.verdict_miss_error(idle));
        }
        if opts.extend_turn_on_busy_health && verdict == DrainHealthVerdict::StillBusy {
            opts.clock.extend_turn_budget(idle);
        }
    }
}
