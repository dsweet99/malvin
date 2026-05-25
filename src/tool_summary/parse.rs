use serde_json::Value;

use super::parse_acp::{
    acp_content_diff_paths, acp_line_range_field, acp_path_field, acp_search_query_field,
    merge_content_diff_paths,
};
use super::types::{
    TOOL_PHASE_DONE, TOOL_PHASE_NAMED_STATUS, TOOL_PHASE_PENDING, TOOL_PHASE_RUNNING,
    TOOL_PHASE_START,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LineRange {
    pub(crate) start: u64,
    pub(crate) end: Option<u64>,
}

pub(crate) struct ParsedToolUpdate {
    pub(crate) phase: u8,
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) title: String,
    pub(crate) status: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) input_path: Option<String>,
    pub(crate) input_line_range: Option<LineRange>,
    pub(crate) search_query: Option<String>,
    pub(crate) raw_output: Option<Value>,
}

pub(crate) fn tool_phase_label(phase: u8, named_status: Option<&str>) -> String {
    match phase {
        TOOL_PHASE_RUNNING => "running".to_string(),
        TOOL_PHASE_DONE => "done".to_string(),
        TOOL_PHASE_PENDING => "pending".to_string(),
        TOOL_PHASE_NAMED_STATUS => named_status.unwrap_or("update").to_string(),
        _ => "start".to_string(),
    }
}

pub(crate) fn phase_for_session_update(session_update: &str, status: Option<&str>) -> Option<u8> {
    match session_update {
        "tool_call" => Some(TOOL_PHASE_START),
        "tool_call_update" => match status {
            Some("in_progress") => Some(TOOL_PHASE_RUNNING),
            Some("completed" | "failed" | "cancelled") => Some(TOOL_PHASE_DONE),
            Some("pending") => Some(TOOL_PHASE_PENDING),
            Some(_) => Some(TOOL_PHASE_NAMED_STATUS),
            None => None,
        },
        _ => None,
    }
}

pub(crate) fn parse_tool_update_fields(
    update: &Value,
    raw_input: Option<&Value>,
    raw_output: Option<&Value>,
) -> (
    Option<String>,
    Option<LineRange>,
    Option<String>,
    Option<Value>,
) {
    let content_diff_paths = acp_content_diff_paths(update);
    let input_path = acp_path_field(raw_input)
        .or_else(|| acp_path_field(raw_output))
        .or_else(|| content_diff_paths.as_ref().and_then(|paths| paths.first().cloned()));
    let input_line_range =
        acp_line_range_field(raw_input).or_else(|| acp_line_range_field(raw_output));
    let search_query =
        acp_search_query_field(raw_input).or_else(|| acp_search_query_field(raw_output));
    let raw_output = merge_content_diff_paths(raw_output, content_diff_paths.as_deref());
    (input_path, input_line_range, search_query, raw_output)
}

pub(crate) fn parse_tool_update_identity(update: &Value) -> Option<(u8, String)> {
    let session_update = update.get("sessionUpdate").and_then(Value::as_str)?;
    let phase = phase_for_session_update(session_update, update.get("status").and_then(Value::as_str))?;
    let id = update.get("toolCallId").and_then(Value::as_str)?.to_string();
    Some((phase, id))
}

pub(crate) fn parse_tool_update_metadata(
    update: &Value,
    raw_input: Option<&Value>,
) -> (String, String, Option<String>, Option<String>) {
    let kind = update
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let title = update
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let status = update.get("status").and_then(Value::as_str).map(str::to_string);
    let command = raw_input
        .and_then(|v| v.get("command"))
        .and_then(Value::as_str)
        .map(str::to_string);
    (kind, title, status, command)
}

pub(crate) fn parse_tool_update(v: &Value) -> Option<ParsedToolUpdate> {
    if v.get("method").and_then(Value::as_str) != Some("session/update") {
        return None;
    }
    let update = v.pointer("/params/update")?;
    let (phase, id) = parse_tool_update_identity(update)?;
    let raw_input = update.get("rawInput");
    let raw_output = update.get("rawOutput");
    let (input_path, input_line_range, search_query, raw_output) =
        parse_tool_update_fields(update, raw_input, raw_output);
    let (kind, title, status, command) = parse_tool_update_metadata(update, raw_input);
    Some(ParsedToolUpdate {
        phase,
        id,
        kind,
        title,
        status,
        command,
        input_path,
        input_line_range,
        search_query,
        raw_output,
    })
}

pub(crate) fn json_number(v: &Value) -> Option<u64> {
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    if let Some(n) = v.as_i64() {
        return u64::try_from(n).ok();
    }
    if let Some(s) = v.as_str() {
        return s.parse().ok();
    }
    None
}
