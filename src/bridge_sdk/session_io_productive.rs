use crate::bridge_protocol::BridgeEvent;

use super::DrainIdleTurn;
use super::stream_log::StreamLog;

pub(crate) fn tools_in_flight(session: &StreamLog) -> bool {
    !session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_empty()
}

pub(crate) fn note_productive_bridge_event(
    _session: &StreamLog,
    turn: &mut DrainIdleTurn,
    ev: &BridgeEvent,
) {
    use crate::sdk_drain_timeout::sdk_drain_idle_max_wait;
    match ev {
        BridgeEvent::ToolCall { phase, .. } if phase == "start" => {
            turn.clock
                .extend_turn_budget(sdk_drain_idle_max_wait(turn.idle()));
        }
        BridgeEvent::Progress { kind, .. } if kind.as_deref() == Some("heartbeat") => {
            turn.clock.extend_turn_budget(turn.idle());
        }
        _ => {}
    }
}
