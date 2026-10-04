use crate::acp::AgentError;

use super::{DrainIdleLabels, DrainIdleTurn, JsonLineSession};

pub(crate) trait TurnProtocol: JsonLineSession {
    type State;
    const WAITING_FOR: &'static str;

    async fn handle(
        &self,
        value: &serde_json::Value,
        state: &mut Self::State,
        turn: &mut DrainIdleTurn,
    ) -> Option<Result<(), AgentError>>;
}

pub(crate) async fn consume_turn<P: TurnProtocol>(
    session: &P,
    mut state: P::State,
) -> Result<(), AgentError> {
    let mut turn = DrainIdleTurn::new();
    loop {
        let value = session.read_json_waiting(P::WAITING_FOR, &mut turn).await?;
        if let Some(result) = session.handle(&value, &mut state, &mut turn).await {
            return result;
        }
        turn.check_max_deadline(DrainIdleLabels {
            prefix: P::BACKEND.drain_idle_prefix(),
            waiting_for: P::WAITING_FOR,
        })?;
    }
}
