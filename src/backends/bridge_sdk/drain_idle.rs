use std::future::Future;

use crate::agent_process::AgentError;

pub(crate) use super::bridge_sdk_drain_idle_health as drain_idle_health;
pub(crate) use super::bridge_sdk_drain_idle_turn as drain_idle_turn;
#[cfg(test)]
pub(crate) use super::bridge_sdk_drain_idle_types::DrainIdleClock;
pub use super::bridge_sdk_drain_idle_types::{
    DrainHealthVerdict, DrainIdleHealthCtx, DrainIdleLabels,
};
pub(crate) use super::bridge_sdk_drain_idle_wait as drain_idle_wait;
use drain_idle_health::sample_drain_health;
pub(crate) use drain_idle_turn::DrainIdleTurn;
pub(crate) use drain_idle_wait::{DrainIdleWaitOpts, await_next_with_idle_using};

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
