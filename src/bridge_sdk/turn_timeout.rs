use std::collections::HashSet;
use std::future::Future;

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::model_id::ModelBackend;

use super::session_io_productive::{note_productive_bridge_event, tools_in_flight};
use super::stream_log::StreamLog;
use super::{DrainIdleHealthCtx, DrainIdleLabels, DrainIdleTurn, await_next_with_idle_in_turn};

#[derive(Debug, Clone, Copy)]
pub(crate) struct TurnTimeoutExtension {
    pub tools_in_flight: fn(&StreamLog) -> bool,
    pub note_productive_event: fn(&StreamLog, &mut DrainIdleTurn, &BridgeEvent),
}

const CURSOR: TurnTimeoutExtension = TurnTimeoutExtension {
    tools_in_flight,
    note_productive_event: note_productive_bridge_event,
};

const PI: TurnTimeoutExtension = TurnTimeoutExtension {
    tools_in_flight,
    note_productive_event: note_productive_bridge_event,
};

const CODEX: TurnTimeoutExtension = TurnTimeoutExtension {
    tools_in_flight,
    note_productive_event: note_productive_bridge_event,
};

#[must_use]
pub(crate) const fn turn_timeout_extension(backend: ModelBackend) -> TurnTimeoutExtension {
    match backend {
        ModelBackend::Cursor => CURSOR,
        ModelBackend::Pi => PI,
        ModelBackend::Codex => CODEX,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct TurnWait<'a> {
    pub backend: ModelBackend,
    pub log: &'a StreamLog,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: &'a HashSet<u32>,
}

impl TurnWait<'_> {
    pub(crate) fn health(&self) -> DrainIdleHealthCtx<'_> {
        DrainIdleHealthCtx {
            process_group_id: self.process_group_id,
            spawn_pid_baseline: self.spawn_pid_baseline,
            tools_in_flight: (turn_timeout_extension(self.backend).tools_in_flight)(self.log),
        }
    }

    pub(crate) fn note_productive_event(&self, turn: &mut DrainIdleTurn, ev: &BridgeEvent) {
        (turn_timeout_extension(self.backend).note_productive_event)(self.log, turn, ev);
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
