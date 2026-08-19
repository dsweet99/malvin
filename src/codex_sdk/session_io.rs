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
    let mut response_text = String::new();
    let mut turn_id = None;
    loop {
        let value = read_json(session).await?;
        if value.get("error").is_some() {
            return Err(AgentError(format!("codex RPC error: {}", value["error"])));
        }
        let method = value.get("method").and_then(|v| v.as_str()).unwrap_or("");
        if method == "turn/started" {
            turn_id = value
                .pointer("/params/turn/id")
                .and_then(|v| v.as_str())
                .map(str::to_owned);
        }
        if method == "item/agentMessage/delta" {
            let event_turn = value.pointer("/params/turnId").and_then(|v| v.as_str());
            if turn_id
                .as_deref()
                .is_some_and(|id| event_turn.is_some_and(|event| event != id))
            {
                continue;
            }
            if let Some(text) = value.pointer("/params/delta").and_then(|v| v.as_str()) {
                response_text.push_str(text);
                crate::bridge_sdk::handle_stream_event(
                    session,
                    &BridgeEvent::Assistant { text: text.into() },
                );
            }
        } else if method == "turn/completed" {
            let event_turn = value.pointer("/params/turn/id").and_then(|v| v.as_str());
            if turn_id
                .as_deref()
                .is_some_and(|id| event_turn.is_some_and(|event| event != id))
            {
                continue;
            }
            let status = value
                .pointer("/params/turn/status")
                .and_then(|v| v.as_str())
                .unwrap_or("completed");
            let result = value
                .pointer("/params/turn/lastAgentMessage")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .or_else(|| (!response_text.is_empty()).then_some(response_text.clone()));
            if let Some(text) = &result {
                *session
                    .last_response
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
                crate::bridge_sdk::feed_do_dm_run_result(text);
            }
            let ev = BridgeEvent::RunDone {
                status: status.into(),
                result,
                usage: None,
                error: None,
                duration_ms: None,
            };
            crate::bridge_sdk::handle_stream_event(session, &ev);
            if crate::bridge_sdk::run_done_status_is_failure(status) {
                let detail = value
                    .pointer("/params/turn/error/message")
                    .and_then(|v| v.as_str())
                    .unwrap_or(status);
                return Err(AgentError(format!("codex turn {detail}")));
            }
            return Ok(());
        }
    }
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
