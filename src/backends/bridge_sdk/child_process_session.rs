use std::future::Future;

use crate::agent_process::AgentError;

use super::bridge_sdk::session_io_productive::tools_in_flight;
use super::bridge_sdk::stdio_child::StdioChild;
use super::bridge_sdk::turn_timeout::TurnTimeoutExtension;
use super::bridge_sdk::{DrainIdleHealthCtx, DrainIdleLabels, DrainIdleTurn, await_next_with_idle_in_turn};

pub(crate) trait ChildProcessSession: Sync {
    fn stdio(&self) -> &StdioChild;
    fn stdio_mut(&mut self) -> &mut StdioChild;
}

impl<T: ChildProcessSession> TurnTimeoutExtension for T {
    fn tools_in_flight(&self) -> bool {
        tools_in_flight(&self.stdio().log)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TurnWait<'a> {
    extension: &'a dyn TurnTimeoutExtension,
    stdio: &'a StdioChild,
}

impl<'a> TurnWait<'a> {
    #[must_use]
    pub(crate) fn of<S: ChildProcessSession>(session: &'a S) -> Self {
        Self {
            extension: session,
            stdio: session.stdio(),
        }
    }

    pub(crate) fn health(&self) -> DrainIdleHealthCtx<'_> {
        DrainIdleHealthCtx {
            process_group_id: self.stdio.process_group_id,
            spawn_pid_baseline: &self.stdio.spawn_pid_baseline,
            tools_in_flight: self.extension.tools_in_flight(),
        }
    }
}

pub(crate) async fn await_turn_event<T, Fut>(
    wait: TurnWait<'_>,
    labels: DrainIdleLabels<'_>,
    read: Fut,
    turn: &mut DrainIdleTurn,
) -> Result<T, AgentError>
where
    Fut: Future<Output = Result<T, AgentError>>,
{
    await_next_with_idle_in_turn(labels, Some(wait.health()), read, turn).await
}
