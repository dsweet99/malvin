use crate::bridge_protocol::BridgeEvent;

use super::DrainIdleTurn;
use super::session::BridgeSession;

pub(super) fn tools_in_flight(session: &BridgeSession) -> bool {
    !session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_empty()
}

/// Extend the drain turn budget on productive Cursor bridge signals.
///
/// Tool starts bump by `2×idle`. Heartbeats bump by `idle` whenever the SDK run is open
/// (alive signal; does not require `tools_in_flight`). Silence for a full idle window
/// remains the hung/quiet bridge failure.
pub(super) fn note_productive_bridge_event(
    _session: &BridgeSession,
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
