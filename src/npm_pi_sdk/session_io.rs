use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use super::session::NpmPiSession;
use crate::acp::AgentError;

static SEQ: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_id() -> String {
    format!("npm-pi-{}", SEQ.fetch_add(1, Ordering::Relaxed))
}

pub(crate) async fn npm_pi_send_prompt(
    session: &NpmPiSession,
    prompt: &str,
) -> Result<(), AgentError> {
    let id = next_id();
    write_json(
        session,
        &serde_json::json!({
            "id": id,
            "type": "prompt",
            "message": prompt,
        }),
    )
    .await?;
    super::session_turn::consume_npm_pi_turn(session, &id).await
}

pub(crate) async fn write_json(
    session: &NpmPiSession,
    value: &serde_json::Value,
) -> Result<(), AgentError> {
    let mut stdin = session.stdin.lock().await;
    stdin
        .write_all(format!("{value}\n").as_bytes())
        .await
        .map_err(|e| AgentError::session_dead(format!("npm pi write: {e}")))?;
    stdin
        .flush()
        .await
        .map_err(|e| AgentError::session_dead(format!("npm pi flush: {e}")))
}

pub(crate) async fn read_json_waiting(
    session: &NpmPiSession,
    waiting_for: &str,
    turn: &mut crate::bridge_sdk::DrainIdleTurn,
) -> Result<serde_json::Value, AgentError> {
    let labels = crate::bridge_sdk::DrainIdleLabels {
        prefix: crate::model_id::ModelBackend::NpmPi.drain_idle_prefix(),
        waiting_for,
    };
    let health = Some(crate::bridge_sdk::DrainIdleHealthCtx {
        process_group_id: session.process_group_id,
        spawn_pid_baseline: &session.spawn_pid_baseline,
        tools_in_flight: crate::bridge_sdk::tools_in_flight(&session.log),
    });
    crate::bridge_sdk::await_next_with_idle_in_turn(labels, health, read_json_line(session), turn)
        .await
}

async fn read_json_line(session: &NpmPiSession) -> Result<serde_json::Value, AgentError> {
    let mut line = String::new();
    let n = {
        let mut out = session.stdout.lock().await;
        out.read_line(&mut line)
            .await
            .map_err(|e| AgentError::session_dead(format!("npm pi read: {e}")))?
    };
    if n == 0 {
        return Err(AgentError::session_dead("npm pi stdout closed"));
    }
    let line = line.trim_end_matches(['\r', '\n']);
    serde_json::from_str(line)
        .map_err(|e| AgentError::session_dead(format!("npm pi JSONL parse: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id_increments() {
        let a = next_id();
        let b = next_id();
        assert_ne!(a, b);
        assert!(a.starts_with("npm-pi-"));
    }
}
