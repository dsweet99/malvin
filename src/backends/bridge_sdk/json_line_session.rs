use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent_process::AgentError;
use crate::config::model_id::ModelBackend;

use super::bridge_sdk::child_process_session::{ChildProcessSession, TurnWait, await_turn_event};
use super::bridge_sdk::{DrainIdleLabels, DrainIdleTurn};

pub(crate) trait JsonLineSession: ChildProcessSession + Sized {
    const BACKEND: ModelBackend;

    async fn write_json(&self, value: &serde_json::Value) -> Result<(), AgentError> {
        let label = Self::BACKEND.wire_label();
        let mut stdin = self.stdio().stdin.lock().await;
        stdin
            .write_all(format!("{value}\n").as_bytes())
            .await
            .map_err(|e| AgentError::session_dead(format!("{label} write: {e}")))?;
        stdin
            .flush()
            .await
            .map_err(|e| AgentError::session_dead(format!("{label} flush: {e}")))
    }

    async fn read_json_waiting(
        &self,
        waiting_for: &str,
        turn: &mut DrainIdleTurn,
    ) -> Result<serde_json::Value, AgentError> {
        let labels = DrainIdleLabels {
            prefix: Self::BACKEND.drain_idle_prefix(),
            waiting_for,
        };
        await_turn_event(TurnWait::of(self), labels, read_json_line(self), turn).await
    }
}

async fn read_json_line<S: JsonLineSession>(session: &S) -> Result<serde_json::Value, AgentError> {
    let label = S::BACKEND.wire_label();
    let mut line = String::new();
    let n = {
        let mut out = session.stdio().stdout.lock().await;
        out.read_line(&mut line)
            .await
            .map_err(|e| AgentError::session_dead(format!("{label} read: {e}")))?
    };
    if n == 0 {
        return Err(AgentError::session_dead(format!("{label} stdout closed")));
    }
    serde_json::from_str(line.trim_end_matches(['\r', '\n']))
        .map_err(|e| AgentError::session_dead(json_parse_error(S::BACKEND, &e)))
}

pub(crate) fn json_parse_error(backend: ModelBackend, err: &serde_json::Error) -> String {
    let parse = backend.spec().wire_parse_label().unwrap_or("JSON");
    format!("{} {parse} parse: {err}", backend.wire_label())
}
