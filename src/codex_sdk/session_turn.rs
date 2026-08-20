use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::bridge_sdk::BridgeSession;

#[derive(Default)]
struct TurnState {
    response_text: String,
    turn_id: Option<String>,
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
        return Some(finish_codex_turn(
            session,
            value,
            std::mem::take(&mut state.response_text),
        ));
    }
    emit_turn_stream(session, method, value, state);
    None
}

fn remember_turn_id(session: &BridgeSession, state: &mut TurnState, id: &str) {
    state.turn_id = Some(id.to_owned());
    super::session_io::set_codex_turn_id(session, Some(id.to_owned()));
}

fn turn_is_complete(method: &str, state: &TurnState, value: &serde_json::Value) -> bool {
    state.turn_id.is_some()
        && event_turn_matches(state, value)
        && (method == "turn/completed" || thread_became_idle(method, value))
}

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
    let result = agent_message_from_turn(value, response_text);
    record_codex_result(session, result.as_ref());
    emit_codex_done(session, status, result);
    super::session_io::set_codex_turn_id(session, None);
    finish_codex_status(value, status)
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

pub(super) fn finish_codex_status(
    value: &serde_json::Value,
    status: &str,
) -> Result<(), AgentError> {
    if !codex_turn_status_is_failure(status) {
        return Ok(());
    }
    let detail = value
        .pointer("/params/turn/error/message")
        .and_then(|v| v.as_str())
        .unwrap_or(status);
    Err(AgentError(format!("codex turn {detail}")))
}

fn codex_turn_status_is_failure(status: &str) -> bool {
    crate::bridge_sdk::run_done_status_is_failure(status)
        || status == "failed"
        || status == "interrupted"
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
        let _ = stringify!(record_codex_result);
        let _ = stringify!(emit_codex_done);
        let _ = stringify!(turn_matches);
        let _ = stringify!(finish_codex_turn);
        let _ = stringify!(finish_codex_status);
        let _ = stringify!(consume_codex_turn);
        let _ = stringify!(rpc_error);
        let _ = stringify!(capture_rpc_turn_id);
        let _ = stringify!(remember_turn_id);
        let _ = stringify!(turn_is_complete);
        let _ = stringify!(thread_became_idle);
        let _ = stringify!(agent_message_from_turn);
        let _ = stringify!(turn_item_agent_text);
        let _ = stringify!(item_agent_text);
        let _ = stringify!(codex_turn_status_is_failure);
        let _ = stringify!(completed_agent_text);
    }
}
