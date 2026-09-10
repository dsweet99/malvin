use super::session::NpmPiSession;
use crate::acp::AgentError;
use crate::bridge_protocol::{BridgeEvent, RunDoneStatus};

#[derive(Default)]
pub(super) struct TurnState {
    pub(super) response_text: String,
    pub(super) usage: Option<serde_json::Value>,
    pub(super) prompt_accepted: bool,
    pub(super) settled: bool,
}

pub(super) async fn consume_npm_pi_turn(
    session: &NpmPiSession,
    prompt_id: &str,
) -> Result<(), AgentError> {
    let mut state = TurnState::default();
    let mut turn = crate::bridge_sdk::DrainIdleTurn::new();
    loop {
        let value = super::session_io::read_json_waiting(session, "npm pi event", &mut turn).await?;
        if let Some(result) = handle_line(session, &value, &mut state, prompt_id).await {
            return result;
        }
        turn.check_max_deadline(crate::bridge_sdk::DrainIdleLabels {
            prefix: crate::model_id::ModelBackend::NpmPi.drain_idle_prefix(),
            waiting_for: "npm pi event",
        })?;
    }
}

async fn handle_line(
    session: &NpmPiSession,
    value: &serde_json::Value,
    state: &mut TurnState,
    prompt_id: &str,
) -> Option<Result<(), AgentError>> {
    let ty = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if ty == "response" {
        return handle_response(value, state, prompt_id);
    }
    if ty == "extension_ui_request" {
        if let Err(e) = super::map_event::auto_reply_extension_ui(session, value).await {
            return Some(Err(e));
        }
        return None;
    }
    for ev in super::map_event::map_npm_pi_event(value, state) {
        if let BridgeEvent::RunDone { .. } = &ev {
            feed_and_handle_run_done(session, &ev);
            return Some(Ok(()));
        }
        crate::bridge_sdk::handle_stream_event(session, &ev);
    }
    if state.settled {
        finish_settled(session, state);
        return Some(Ok(()));
    }
    None
}

fn handle_response(
    value: &serde_json::Value,
    state: &mut TurnState,
    prompt_id: &str,
) -> Option<Result<(), AgentError>> {
    let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if id != prompt_id {
        return None;
    }
    let success = value
        .get("success")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !success {
        let err = value
            .get("error")
            .map_or_else(|| value.to_string(), ToString::to_string);
        return Some(Err(AgentError(format!("npm pi prompt rejected: {err}"))));
    }
    state.prompt_accepted = true;
    None
}

fn finish_settled(session: &NpmPiSession, state: &mut TurnState) {
    let text = std::mem::take(&mut state.response_text);
    if !text.is_empty() {
        *session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
    }
    let ev = BridgeEvent::RunDone {
        status: RunDoneStatus::Finished,
        result: (!text.is_empty()).then_some(text),
        usage: state.usage.take(),
        error: None,
        duration_ms: None,
    };
    if let BridgeEvent::RunDone { usage: Some(u), .. } = &ev {
        crate::bridge_sdk::record_sdk_usage(session.timing.as_ref(), u);
    }
    feed_and_handle_run_done(session, &ev);
}

fn feed_and_handle_run_done(session: &NpmPiSession, ev: &BridgeEvent) {
    feed_run_done_dm(ev);
    crate::bridge_sdk::handle_stream_event(session, ev);
}

fn feed_run_done_dm(ev: &BridgeEvent) {
    if let BridgeEvent::RunDone {
        result: Some(text), ..
    } = ev
    {
        crate::bridge_sdk::feed_do_dm_run_result(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{
        DM_END, DM_START, enable_stdout_capture, set_do_dm_stdout_mode, take_captured_stdout,
    };

    #[test]
    fn prompt_reject_maps_error() {
        let mut state = TurnState::default();
        let value = serde_json::json!({
            "type": "response",
            "id": "req-1",
            "success": false,
            "error": "busy"
        });
        let err = handle_response(&value, &mut state, "req-1")
            .expect("handled")
            .expect_err("reject");
        assert!(err.message.contains("busy"));
    }

    #[test]
    fn feed_run_done_dm_extracts_fenced_body() {
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        let ev = BridgeEvent::RunDone {
            status: RunDoneStatus::Finished,
            result: Some(format!("{DM_START}\nHello.\n{DM_END}")),
            usage: None,
            error: None,
            duration_ms: None,
        };
        feed_run_done_dm(&ev);
        let out = take_captured_stdout();
        set_do_dm_stdout_mode(false);
        assert_eq!(out.trim(), "Hello.");
    }
}
