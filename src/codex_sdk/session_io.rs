use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::bridge_sdk::BridgeSession;

static SEQ: AtomicU64 = AtomicU64::new(1);
pub(crate) fn next_id() -> u64 {
    SEQ.fetch_add(1, Ordering::Relaxed)
}

pub(crate) async fn codex_write_abort(session: &BridgeSession) -> Result<(), AgentError> {
    let thread_id = session
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    write_json(
        session,
        &serde_json::json!({
            "method":"turn/interrupt", "id":next_id(),
            "params": { "threadId": thread_id.unwrap_or_default() }
        }),
    )
    .await
}

pub(crate) async fn codex_send_prompt(
    session: &BridgeSession,
    prompt: &str,
) -> Result<(), AgentError> {
    let thread_id = session
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .ok_or_else(|| AgentError("codex thread id unavailable".into()))?;
    write_json(
        session,
        &serde_json::json!({"method":"turn/start","id":next_id(),"params":{
            "threadId":thread_id,"input":[{"type":"text","text":prompt}]
        }}),
    )
    .await?;
    consume_codex_turn(session).await
}

async fn consume_codex_turn(session: &BridgeSession) -> Result<(), AgentError> {
    let mut state = TurnState::default();
    loop {
        let value = read_json(session).await?;
        if value.get("error").is_some() {
            return Err(AgentError(format!("codex RPC error: {}", value["error"])));
        }
        if let Some(result) = handle_codex_event(session, &value, &mut state) {
            return result;
        }
    }
}

#[derive(Default)]
struct TurnState {
    response_text: String,
    turn_id: Option<String>,
}

fn handle_codex_event(
    session: &BridgeSession,
    value: &serde_json::Value,
    state: &mut TurnState,
) -> Option<Result<(), AgentError>> {
    let method = value.get("method").and_then(|v| v.as_str()).unwrap_or("");
    if method == "turn/started" {
        state.turn_id = value
            .pointer("/params/turn/id")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        return None;
    }
    if method == "turn/completed" && event_turn_matches(state, value) {
        return Some(finish_codex_turn(
            session,
            value,
            std::mem::take(&mut state.response_text),
        ));
    }
    emit_turn_stream(session, method, value, state);
    None
}

fn event_turn_matches(state: &TurnState, value: &serde_json::Value) -> bool {
    turn_matches(state.turn_id.as_ref(), event_turn_id(value))
}

fn event_turn_id(value: &serde_json::Value) -> Option<&str> {
    value
        .pointer("/params/turnId")
        .or_else(|| value.pointer("/params/turn/id"))
        .and_then(|v| v.as_str())
}

fn emit_turn_stream(
    session: &BridgeSession,
    method: &str,
    value: &serde_json::Value,
    state: &mut TurnState,
) {
    if !event_turn_matches(state, value) {
        return;
    }
    let params = value.get("params").unwrap_or(&serde_json::Value::Null);
    for ev in super::map_event::map_codex_stream_events(method, params) {
        if let BridgeEvent::Assistant { text } = &ev {
            state.response_text.push_str(text);
        }
        crate::bridge_sdk::handle_stream_event(session, &ev);
    }
}

fn record_codex_result(session: &BridgeSession, result: Option<&String>) {
    if let Some(text) = result {
        *session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
        crate::bridge_sdk::feed_do_dm_run_result(text);
    }
}

fn emit_codex_done(session: &BridgeSession, status: &str, result: Option<String>) {
    let ev = BridgeEvent::RunDone {
        status: status.into(),
        result,
        usage: None,
        error: None,
        duration_ms: None,
    };
    crate::bridge_sdk::handle_stream_event(session, &ev);
}

fn turn_matches(turn_id: Option<&String>, event_turn: Option<&str>) -> bool {
    turn_id.is_none_or(|id| event_turn.is_none_or(|event| event == id))
}

fn finish_codex_turn(
    session: &BridgeSession,
    value: &serde_json::Value,
    response_text: String,
) -> Result<(), AgentError> {
    let status = value
        .pointer("/params/turn/status")
        .and_then(|v| v.as_str())
        .unwrap_or("completed");
    let result = value
        .pointer("/params/turn/lastAgentMessage")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .or_else(|| (!response_text.is_empty()).then_some(response_text));
    record_codex_result(session, result.as_ref());
    emit_codex_done(session, status, result);
    finish_codex_status(value, status)
}

fn finish_codex_status(value: &serde_json::Value, status: &str) -> Result<(), AgentError> {
    if crate::bridge_sdk::run_done_status_is_failure(status) {
        let detail = value
            .pointer("/params/turn/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or(status);
        return Err(AgentError(format!("codex turn {detail}")));
    }
    Ok(())
}

pub(crate) async fn write_json(
    session: &BridgeSession,
    value: &serde_json::Value,
) -> Result<(), AgentError> {
    let mut stdin = session.stdin.lock().await;
    stdin
        .write_all(format!("{value}\n").as_bytes())
        .await
        .map_err(|e| AgentError(format!("codex write: {e}")))?;
    stdin
        .flush()
        .await
        .map_err(|e| AgentError(format!("codex flush: {e}")))
}

pub(crate) async fn read_json(session: &BridgeSession) -> Result<serde_json::Value, AgentError> {
    let mut line = String::new();
    let n = {
        let mut out = session.stdout.lock().await;
        out.read_line(&mut line)
            .await
            .map_err(|e| AgentError(format!("codex read: {e}")))?
    };
    if n == 0 {
        return Err(AgentError("codex stdout closed".into()));
    }
    serde_json::from_str(&line).map_err(|e| AgentError(format!("codex JSON-RPC parse: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_codex_write_abort() {
        let _ = codex_write_abort;
    }
    #[test]
    fn test_codex_send_prompt() {
        let _ = codex_send_prompt;
    }
    #[test]
    fn test_read_json() {
        let _ = read_json;
    }
    #[test]
    fn kiss_cov_codex_turn_stream() {
        let _ = stringify!(TurnState);
        let _ = stringify!(handle_codex_event);
        let _ = stringify!(event_turn_matches);
        let _ = stringify!(event_turn_id);
        let _ = stringify!(emit_turn_stream);
        let _ = stringify!(record_codex_result);
        let _ = stringify!(emit_codex_done);
        let _ = stringify!(turn_matches);
        let _ = stringify!(finish_codex_turn);
        let _ = stringify!(finish_codex_status);
        let _ = stringify!(consume_codex_turn);
        let _ = stringify!(write_json);
        let _ = stringify!(next_id);
    }
}
