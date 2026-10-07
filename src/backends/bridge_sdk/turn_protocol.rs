use crate::agent_process::AgentError;

use super::bridge_sdk::{DrainIdleTurn, JsonLineSession};

pub(crate) trait TurnProtocol: JsonLineSession {
    type State;
    const WAITING_FOR: &'static str;

    async fn handle(
        &self,
        value: &serde_json::Value,
        state: &mut Self::State,
    ) -> Option<Result<(), AgentError>>;
}

pub(crate) async fn consume_turn<P: TurnProtocol>(
    session: &P,
    mut state: P::State,
) -> Result<(), AgentError> {
    let mut turn = DrainIdleTurn::new();
    loop {
        let value = session.read_json_waiting(P::WAITING_FOR, &mut turn).await?;
        if let Some(result) = session.handle(&value, &mut state).await {
            return result;
        }
    }
}
