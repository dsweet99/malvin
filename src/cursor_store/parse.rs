use serde_json::Value;

use crate::tool_summary::acp_line_range_field;

use super::types::ToolCallArgs;

pub(crate) fn tool_call_path(args: Option<&Value>) -> Option<String> {
    args.and_then(|a| {
        a.get("path")
            .or_else(|| a.get("filePath"))
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

pub(crate) fn parse_tool_call_item(item: &Value) -> Option<(String, ToolCallArgs)> {
    if item.get("type").and_then(Value::as_str) != Some("tool-call") {
        return None;
    }
    let id = item.get("toolCallId").and_then(Value::as_str)?.to_string();
    let args = item.get("args");
    Some((
        id,
        ToolCallArgs {
            path: tool_call_path(args),
            line_range: acp_line_range_field(args),
        },
    ))
}

pub fn parse_tool_call_args_from_blob(data: &str) -> Vec<(String, ToolCallArgs)> {
    if !data.contains("tool-call") {
        return Vec::new();
    }
    let Ok(v) = serde_json::from_str::<Value>(data) else {
        return Vec::new();
    };
    let Some(items) = v.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };
    items.iter().filter_map(parse_tool_call_item).collect()
}
