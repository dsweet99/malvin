use super::session::NpmPiSession;
use crate::acp::AgentError;
use crate::bridge_protocol::{BridgeEvent, RunDoneStatus};
use crate::bridge_sdk::{DrainIdleTurn, TurnProtocol};

#[derive(Default)]
pub(crate) struct TurnState {
    pub(super) prompt_id: String,
    pub(super) response_text: String,
    pub(super) usage: Option<serde_json::Value>,
    pub(super) prompt_accepted: bool,
    pub(super) settled: bool,
    pub(super) output_cap: Option<u64>,
    pub(super) end_error: Option<AgentError>,
}

impl TurnProtocol for NpmPiSession {
    type State = TurnState;
    const WAITING_FOR: &'static str = "npm pi event";

    async fn handle(
        &self,
        value: &serde_json::Value,
        state: &mut TurnState,
        turn: &mut DrainIdleTurn,
    ) -> Option<Result<(), AgentError>> {
        handle_line(self, value, state, turn).await
    }
}

pub(super) async fn consume_npm_pi_turn(
    session: &NpmPiSession,
    prompt_id: &str,
) -> Result<(), AgentError> {
    let state = TurnState {
        prompt_id: prompt_id.to_owned(),
        output_cap: session.output_cap,
        ..TurnState::default()
    };
    crate::bridge_sdk::consume_turn(session, state).await
}

pub(super) fn feed_mapped_bridge_events(
    session: &NpmPiSession,
    turn: &mut crate::bridge_sdk::DrainIdleTurn,
    events: &[BridgeEvent],
) {
    for ev in events {
        crate::bridge_sdk::TurnTimeoutExtension::note_productive_event(session, turn, ev);
        if let BridgeEvent::Step { .. } = ev {
            crate::bridge_sdk::note_sdk_step(session.timing.as_ref());
        }
        crate::bridge_sdk::handle_stream_event(session, ev);
    }
}

async fn handle_line(
    session: &NpmPiSession,
    value: &serde_json::Value,
    state: &mut TurnState,
    drain: &mut DrainIdleTurn,
) -> Option<Result<(), AgentError>> {
    let ty = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
    if ty == "response" {
        return handle_response(value, state);
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
        feed_mapped_bridge_events(session, drain, std::slice::from_ref(&ev));
    }
    if state.settled {
        return Some(finish_settled(session, state));
    }
    None
}

fn handle_response(
    value: &serde_json::Value,
    state: &mut TurnState,
) -> Option<Result<(), AgentError>> {
    let id = value.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if id != state.prompt_id {
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

fn finish_settled(session: &NpmPiSession, state: &mut TurnState) -> Result<(), AgentError> {
    let end_error = state.end_error.take();
    let text = std::mem::take(&mut state.response_text);
    if !text.is_empty() {
        *session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = text.clone();
    }
    let ev = BridgeEvent::RunDone {
        status: if end_error.is_some() {
            RunDoneStatus::Error
        } else {
            RunDoneStatus::Finished
        },
        result: (!text.is_empty()).then_some(text),
        usage: usage_after_portkey(session, state.usage.take()),
        error: end_error.as_ref().map(|e| e.message.clone()),
        duration_ms: None,
    };
    if let BridgeEvent::RunDone { usage: Some(u), .. } = &ev {
        crate::bridge_sdk::record_sdk_usage(session.timing.as_ref(), u);
    }
    feed_and_handle_run_done(session, &ev);
    end_error.map_or(Ok(()), Err)
}

fn usage_after_portkey(
    session: &NpmPiSession,
    usage: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    let mut usage = usage?;
    if let Some((provider, model)) = &session.pi_model {
        crate::pi_sdk::apply_portkey_cost_usd(provider, model, &mut usage);
    }
    Some(usage)
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
        let mut state = TurnState {
            prompt_id: "req-1".into(),
            ..TurnState::default()
        };
        let value = serde_json::json!({
            "type": "response",
            "id": "req-1",
            "success": false,
            "error": "busy"
        });
        let err = handle_response(&value, &mut state)
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
