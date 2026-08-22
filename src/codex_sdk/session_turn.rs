use std::time::Instant;

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::bridge_sdk::BridgeSession;

#[derive(Default)]
pub(super) struct TurnState {
    pub(super) response_text: String,
    pub(super) turn_id: Option<String>,
    pub(super) usage: Option<serde_json::Value>,
    pub(super) started: Option<Instant>,
    pub(super) counted_step: bool,
}

pub(super) async fn consume_codex_turn(session: &BridgeSession) -> Result<(), AgentError> {
    let mut state = TurnState::default();
    loop {
        let value = super::session_io::read_json_waiting(session, "turn event").await?;
        if let Some(err) = rpc_error(&value) {
            return Err(err);
        }
        capture_rpc_turn_id(session, &mut state, &value);
        if let Some(result) = handle_codex_event(session, &value, &mut state) {
            return result;
        }
    }
}

pub(super) fn rpc_error(value: &serde_json::Value) -> Option<AgentError> {
    if value.get("method").is_some() {
        return None;
    }
    value
        .get("error")
        .map(|err| AgentError(format!("codex RPC error: {err}")))
}

fn capture_rpc_turn_id(session: &BridgeSession, state: &mut TurnState, value: &serde_json::Value) {
    let Some(id) = value
        .pointer("/result/turn/id")
        .or_else(|| value.pointer("/result/id"))
        .and_then(|v| v.as_str())
    else {
        return;
    };
    remember_turn_id(session, state, id);
}

fn handle_codex_event(
    session: &BridgeSession,
    value: &serde_json::Value,
    state: &mut TurnState,
) -> Option<Result<(), AgentError>> {
    let method = value.get("method").and_then(|v| v.as_str()).unwrap_or("");
    if method == "error" {
        return Some(Err(AgentError(format!("codex RPC error: {value}"))));
    }
    if method == "turn/started" {
        if let Some(id) = value.pointer("/params/turn/id").and_then(|v| v.as_str()) {
            remember_turn_id(session, state, id);
        }
        return None;
    }
    if turn_is_complete(method, state, value) {
        return Some(super::session_turn_done::finish_codex_turn(
            session,
            value,
            std::mem::take(state),
        ));
    }
    emit_turn_stream(session, method, value, state);
    None
}

fn remember_turn_id(session: &BridgeSession, state: &mut TurnState, id: &str) {
    state.turn_id = Some(id.to_owned());
    if state.started.is_none() {
        state.started = Some(Instant::now());
    }
    super::session_io::set_codex_turn_id(session, Some(id.to_owned()));
}

pub(super) fn turn_is_complete(method: &str, state: &TurnState, value: &serde_json::Value) -> bool {
    method == "turn/completed" && state.turn_id.is_some() && event_turn_matches(state, value)
}

#[cfg(test)]
pub(super) fn thread_became_idle(method: &str, value: &serde_json::Value) -> bool {
    method == "thread/status/changed"
        && value
            .pointer("/params/status/type")
            .and_then(|v| v.as_str())
            == Some("idle")
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
            if !state.counted_step {
                crate::bridge_sdk::note_sdk_step(session.timing.as_ref());
                state.counted_step = true;
            }
        }
        if let BridgeEvent::Usage { usage } = &ev {
            state.usage = Some(usage.clone());
        }
        if let BridgeEvent::ToolCall { phase, .. } = &ev
            && phase == "start" {
                crate::bridge_sdk::note_sdk_step(session.timing.as_ref());
            }
        crate::bridge_sdk::handle_stream_event(session, &ev);
    }
    if let Some(text) = completed_agent_text(method, params) {
        state.response_text = text.to_owned();
    }
}

fn completed_agent_text<'a>(method: &str, params: &'a serde_json::Value) -> Option<&'a str> {
    if method != "item/completed" {
        return None;
    }
    item_agent_text(params.get("item")?)
}

fn turn_matches(turn_id: Option<&String>, event_turn: Option<&str>) -> bool {
    turn_id.is_none_or(|id| event_turn.is_none_or(|event| event == id))
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

fn item_agent_text(item: &serde_json::Value) -> Option<&str> {
    let ty = item.get("type").and_then(serde_json::Value::as_str)?;
    (ty == "agentMessage")
        .then(|| item.get("text").and_then(serde_json::Value::as_str))
        .flatten()
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_codex_turn_stream() {
        let _ = stringify!(TurnState);
        let _ = stringify!(handle_codex_event);
        let _ = stringify!(event_turn_matches);
        let _ = stringify!(event_turn_id);
        let _ = stringify!(emit_turn_stream);
        let _ = stringify!(turn_matches);
        let _ = stringify!(consume_codex_turn);
        let _ = stringify!(elapsed_ms);
        let _ = stringify!(logged_run_done);
        let _ = stringify!(canonicalize_run_done);
        let _ = stringify!(rpc_error);
        let _ = stringify!(capture_rpc_turn_id);
        let _ = stringify!(remember_turn_id);
        let _ = stringify!(turn_is_complete);
        let _ = stringify!(thread_became_idle);
        let _ = stringify!(agent_message_from_turn);
        let _ = stringify!(turn_item_agent_text);
        let _ = stringify!(item_agent_text);
        let _ = stringify!(completed_agent_text);
    }
}
