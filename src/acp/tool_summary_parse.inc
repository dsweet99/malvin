struct ParsedToolUpdate {
    phase: u8,
    id: String,
    kind: String,
    title: String,
    status: Option<String>,
    command: Option<String>,
    raw_output: Option<Value>,
}

const TOOL_PHASE_START: u8 = 0;
const TOOL_PHASE_RUNNING: u8 = 1;
const TOOL_PHASE_DONE: u8 = 2;
const TOOL_PHASE_PENDING: u8 = 3;

const fn tool_phase_label(phase: u8) -> &'static str {
    match phase {
        TOOL_PHASE_RUNNING => "running",
        TOOL_PHASE_DONE => "done",
        TOOL_PHASE_PENDING => "pending",
        _ => "start",
    }
}

fn phase_for_session_update(session_update: &str, status: Option<&str>) -> Option<u8> {
    match session_update {
        "tool_call" => Some(TOOL_PHASE_START),
        "tool_call_update" => Some(match status {
            Some("in_progress") => TOOL_PHASE_RUNNING,
            Some("completed" | "failed" | "cancelled") => TOOL_PHASE_DONE,
            Some("pending") => TOOL_PHASE_PENDING,
            _ => TOOL_PHASE_RUNNING,
        }),
        _ => None,
    }
}

fn parse_tool_update(v: &Value) -> Option<ParsedToolUpdate> {
    if v.get("method").and_then(Value::as_str) != Some("session/update") {
        return None;
    }
    let update = v.pointer("/params/update")?;
    let session_update = update.get("sessionUpdate").and_then(Value::as_str)?;
    let phase = phase_for_session_update(session_update, update.get("status").and_then(Value::as_str))?;
    let id = update.get("toolCallId").and_then(Value::as_str)?.to_string();
    Some(ParsedToolUpdate {
        phase,
        id,
        kind: update
            .get("kind")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        title: update
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        status: update.get("status").and_then(Value::as_str).map(str::to_string),
        command: update
            .pointer("/rawInput/command")
            .and_then(Value::as_str)
            .map(str::to_string),
        raw_output: update.get("rawOutput").cloned(),
    })
}
