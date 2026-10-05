use crate::bridge_protocol::BridgeEvent;

use super::DrainIdleTurn;

pub(crate) trait TurnTimeoutExtension: Sync {
    fn tools_in_flight(&self) -> bool;
    fn note_productive_event(&self, turn: &mut DrainIdleTurn, ev: &BridgeEvent);
}
