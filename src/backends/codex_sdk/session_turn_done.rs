use std::time::Instant;

use super::codex_sdk::session::CodexSession;
use crate::agent_process::AgentError;
use crate::backends::cursor_sdk::protocol::{BridgeEvent, RunDoneStatus};

#[derive(Default)]
pub(crate) struct TurnState {
    pub(super) response_text: String,
    pub(super) turn_id: Option<String>,
    pub(super) usage: Option<serde_json::Value>,
    pub(super) usage_is_cumulative: bool,
    pub(super) started: Option<Instant>,
    pub(super) counted_step: bool,
}

pub(super) fn finish_codex_turn(
    session: &CodexSession,
    value: &serde_json::Value,
    state: TurnState,
) -> Result<(), AgentError> {
    let usage_is_cumulative =
        state.usage_is_cumulative || super::codex_sdk::map_event_usage::has_cumulative_usage(value);
    let ev = run_done_from_turn(value, state)?;
    if let BridgeEvent::RunDone { usage: Some(u), .. } = &ev {
        let usage = if usage_is_cumulative {
            let mut total = session
                .thread_usage_total
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            super::codex_sdk::map_event_usage::cumulative_usage_delta(&mut total, u)
        } else {
            u.clone()
        };
        crate::backends::bridge_sdk::record_sdk_usage(session.timing.as_ref(), &usage);
    }
    if let BridgeEvent::RunDone {
        result: Some(text), ..
    } = &ev
    {
        record_codex_result(session, text);
    }
    crate::backends::bridge_sdk::handle_stream_event(session, &ev);
    super::codex_sdk::session::set_codex_turn_id(session, None);
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
            .or_else(|| super::codex_sdk::map_event_usage::usage_from_turn(value)),
        error: turn_error_message(value, status),
        duration_ms: turn_duration_ms(value).or_else(|| elapsed_ms(state.started)),
    })
}

pub(super) fn finish_codex_status(
    value: &serde_json::Value,
    status: impl Into<RunDoneStatus>,
) -> Result<(), AgentError> {
    let status = status.into();
    if status == RunDoneStatus::Unknown {
        let err = value
            .pointer("/params/turn/error/message")
            .and_then(|v| v.as_str());
        tracing::warn!(
            error = err,
            "codex run_done unknown status; surfacing turn error"
        );
    }
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
    crate::backends::bridge_sdk::feed_do_dm_run_result(text);
}

fn turn_error_message(value: &serde_json::Value, status: RunDoneStatus) -> Option<String> {
    status.is_failure().then(|| {
        let error = value.pointer("/params/turn/error");
        if let Some(error) = error.filter(|e| !e["codexErrorInfo"].is_null()) {
            return error.to_string();
        }
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

pub(super) fn agent_message_from_turn(
    value: &serde_json::Value,
    response_text: String,
) -> Option<String> {
    turn_item_agent_text(value).or_else(|| (!response_text.is_empty()).then_some(response_text))
}

fn turn_item_agent_text(value: &serde_json::Value) -> Option<String> {
    let items = value.pointer("/params/turn/items")?.as_array()?;
    let texts: Vec<&str> = items.iter().filter_map(item_agent_text).collect();
    (!texts.is_empty()).then(|| texts.join(""))
}

pub(super) fn item_agent_text(item: &serde_json::Value) -> Option<&str> {
    let ty = item.get("type").and_then(serde_json::Value::as_str)?;
    (ty == "agentMessage")
        .then(|| item.get("text").and_then(serde_json::Value::as_str))
        .flatten()
        .filter(|text| !text.is_empty())
}
