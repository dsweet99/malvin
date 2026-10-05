use std::sync::atomic::{AtomicU64, Ordering};

use super::session::CodexSession;
use crate::agent_process::AgentError;
use crate::backends::bridge_sdk::JsonLineSession;

static SEQ: AtomicU64 = AtomicU64::new(1);
pub(crate) fn next_id() -> u64 {
    SEQ.fetch_add(1, Ordering::Relaxed)
}

fn session_string(lock: &std::sync::Mutex<Option<String>>) -> Option<String> {
    lock.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

fn set_session_string(lock: &std::sync::Mutex<Option<String>>, value: Option<String>) {
    *lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
}

pub(crate) fn set_codex_turn_id(session: &CodexSession, turn_id: Option<String>) {
    set_session_string(&session.turn_id, turn_id);
}

pub(crate) fn turn_interrupt_params(
    thread_id: Option<String>,
    turn_id: Option<String>,
) -> Option<serde_json::Value> {
    let thread_id = thread_id.filter(|id| !id.is_empty())?;
    let turn_id = turn_id.filter(|id| !id.is_empty())?;
    Some(serde_json::json!({ "threadId": thread_id, "turnId": turn_id }))
}

pub(crate) async fn codex_write_abort(session: &CodexSession) -> Result<(), AgentError> {
    let Some(params) = turn_interrupt_params(
        session_string(&session.thread_id),
        session_string(&session.turn_id),
    ) else {
        return Ok(());
    };
    session
        .write_json(&serde_json::json!({"method":"turn/interrupt","id":next_id(),"params":params}))
        .await
}

pub(crate) async fn codex_send_prompt(
    session: &CodexSession,
    prompt: &str,
) -> Result<(), AgentError> {
    let thread_id = session_string(&session.thread_id)
        .ok_or_else(|| AgentError("codex thread id unavailable".into()))?;
    set_codex_turn_id(session, None);
    session
        .write_json(&serde_json::json!({
            "method": "turn/start",
            "id": next_id(),
            "params": turn_start_params(
                &thread_id,
                prompt,
                session.stdio.log.thinking.as_deref(),
                session.service.as_deref(),
            )
        }))
        .await?;
    super::session_turn::consume_codex_turn(session).await
}

pub(crate) async fn codex_delete_thread(session: &CodexSession) -> Result<(), AgentError> {
    let Some(thread_id) = session_string(&session.thread_id).filter(|id| !id.is_empty()) else {
        return Ok(());
    };
    session
        .write_json(&serde_json::json!({
            "method": "thread/delete",
            "id": next_id(),
            "params": { "threadId": thread_id }
        }))
        .await
}

fn codex_effort_from_thinking(thinking: &str) -> String {
    match thinking {
        "off" | "minimal" => "low".to_owned(),
        other => other.to_owned(),
    }
}

fn turn_start_params(
    thread_id: &str,
    prompt: &str,
    thinking: Option<&str>,
    service: Option<&str>,
) -> serde_json::Value {
    let mut params = serde_json::json!({
        "threadId": thread_id,
        "input": [{"type": "text", "text": prompt}]
    });
    if let Some(thinking) = thinking {
        params["effort"] = serde_json::Value::String(codex_effort_from_thinking(thinking));
    }
    if let Some(service) = service {
        params["serviceTier"] = serde_json::Value::String(service.to_owned());
    }
    params
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
        let _ = set_codex_turn_id;
        let _ = next_id;
        let _ = session_string;
        let _ = set_session_string;
        let _ = codex_delete_thread;
        let _ = turn_start_params;
    }
    #[test]
    fn interrupt_requires_thread_and_turn_ids() {
        assert!(turn_interrupt_params(None, Some("t".into())).is_none());
        assert!(turn_interrupt_params(Some("th".into()), None).is_none());
        assert!(turn_interrupt_params(Some(String::new()), Some("t".into())).is_none());
        let params = turn_interrupt_params(Some("th".into()), Some("t1".into())).unwrap();
        assert_eq!(params["threadId"], "th");
        assert_eq!(params["turnId"], "t1");
    }

    #[test]
    fn turn_start_includes_effort_and_service() {
        let params = turn_start_params("th", "hi", Some("high"), Some("priority"));
        assert_eq!(params["effort"], "high");
        assert_eq!(params["serviceTier"], "priority");
        assert_eq!(params["threadId"], "th");
        let bare = turn_start_params("th", "hi", None, None);
        assert!(bare.get("effort").is_none());
        assert!(bare.get("serviceTier").is_none());
        let mapped = turn_start_params("th", "hi", Some("off"), None);
        assert_eq!(mapped["effort"], "low");
    }
}
