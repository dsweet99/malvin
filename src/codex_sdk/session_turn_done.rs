use crate::acp::AgentError;
use crate::bridge_protocol::{BridgeEvent, RunDoneStatus};
use super::session::CodexSession;

use super::session_turn::{TurnState, agent_message_from_turn};

pub(super) fn finish_codex_turn(
    session: &CodexSession,
    value: &serde_json::Value,
    state: TurnState,
) -> Result<(), AgentError> {
    let ev = run_done_from_turn(value, state)?;
    if let BridgeEvent::RunDone { usage: Some(u), .. } = &ev {
        crate::bridge_sdk::record_sdk_usage(session.timing.as_ref(), u);
    }
    if let BridgeEvent::RunDone {
        result: Some(text), ..
    } = &ev
    {
        record_codex_result(session, text);
    }
    crate::bridge_sdk::handle_stream_event(session, &ev);
    super::session_io::set_codex_turn_id(session, None);
    let status = match &ev {
        BridgeEvent::RunDone { status, .. } => *status,
        _ => RunDoneStatus::Finished,
    };
    finish_codex_status(value, status)
}

pub(super) fn run_done_from_turn(
    value: &serde_json::Value,
    state: TurnState,
) -> Result<BridgeEvent, AgentError> {
    let raw = value
        .pointer("/params/turn/status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError("codex turn completed without status".into()))?;
    let status = RunDoneStatus::from_raw(raw);
    Ok(BridgeEvent::RunDone {
        status,
        result: agent_message_from_turn(value, state.response_text),
        usage: state
            .usage
            .or_else(|| super::map_event_usage::usage_from_turn(value)),
        error: turn_error_message(value, status),
        duration_ms: turn_duration_ms(value).or_else(|| elapsed_ms(state.started)),
    })
}

pub(super) fn finish_codex_status(
    value: &serde_json::Value,
    status: impl Into<RunDoneStatus>,
) -> Result<(), AgentError> {
    let status = status.into();
    let Some(detail) = turn_error_message(value, status) else {
        return Ok(());
    };
    Err(AgentError(format!("codex turn {detail}")))
}

pub(super) fn turn_duration_ms(value: &serde_json::Value) -> Option<u64> {
    json_u64(value.pointer("/params/turn/durationMs")).or_else(|| {
        let start = json_i64(value.pointer("/params/turn/startedAt"))?;
        let end = json_i64(value.pointer("/params/turn/completedAt"))?;
        u64::try_from((end - start).max(0).saturating_mul(1000)).ok()
    })
}

fn elapsed_ms(started: Option<std::time::Instant>) -> Option<u64> {
    started.and_then(|s| u64::try_from(s.elapsed().as_millis()).ok())
}

fn record_codex_result(session: &CodexSession, text: &str) {
    *session
        .last_response
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = text.to_owned();
    crate::bridge_sdk::feed_do_dm_run_result(text);
}

fn turn_error_message(value: &serde_json::Value, status: RunDoneStatus) -> Option<String> {
    status.is_failure().then(|| {
        value
            .pointer("/params/turn/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| status.as_str())
            .to_owned()
    })
}

fn json_u64(value: Option<&serde_json::Value>) -> Option<u64> {
    let v = value?;
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    u64::try_from(v.as_i64()?).ok()
}

fn json_i64(value: Option<&serde_json::Value>) -> Option<i64> {
    let v = value?;
    if let Some(n) = v.as_i64() {
        return Some(n);
    }
    i64::try_from(v.as_u64()?).ok()
}
