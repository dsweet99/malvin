use crate::bridge_protocol::BridgeEvent;

use super::session::BridgeSession;
use super::DrainIdleTurn;

pub(super) fn tools_in_flight(session: &BridgeSession) -> bool {
    !session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_empty()
}

pub(super) fn note_productive_bridge_event(
    session: &BridgeSession,
    turn: &mut DrainIdleTurn,
    ev: &BridgeEvent,
) {
    use crate::sdk_drain_timeout::sdk_drain_idle_max_wait;
    match ev {
        BridgeEvent::ToolCall { phase, .. } if phase == "start" => {
            turn.clock
                .extend_turn_budget(sdk_drain_idle_max_wait(turn.idle()));
        }
        BridgeEvent::Progress { kind, .. } if kind.as_deref() == Some("heartbeat")
            && tools_in_flight(session) =>
        {
            turn.clock.extend_turn_budget(turn.idle());
        }
        _ => {}
    }
}
