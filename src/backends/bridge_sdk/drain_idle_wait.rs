use std::future::Future;
use std::time::Duration;

use crate::agent_process::AgentError;

use super::bridge_sdk::drain_idle::{DrainHealthVerdict, DrainIdleClock, DrainIdleLabels};

pub(crate) struct DrainIdleWaitOpts<'a> {
    pub labels: DrainIdleLabels<'a>,
    pub clock: &'a mut DrainIdleClock,
    pub tools_in_flight: bool,
}

impl DrainIdleWaitOpts<'_> {
    fn silence_error(&self, idle: Duration) -> AgentError {
        self.labels.silence_error_detail(idle, self.tools_in_flight)
    }
}

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
            return Err(opts.silence_error(idle));
        };
        if let Ok(result) = tokio::time::timeout(slice, read.as_mut()).await {
            return result;
        }
        let verdict = tokio::select! {
            result = read.as_mut() => return result,
            verdict = health_sampler(slice) => verdict,
        };
        if opts.clock.apply_verdict(verdict).is_err() {
            return Err(opts.silence_error(idle));
        }
    }
}
