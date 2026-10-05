use serde_json::Value;

use crate::tool_summary::{TOOL_DISPLAY_MAX_WIDTH, shorten_middle};

pub(super) fn tool_name_summary(ty: &str, item: &Value) -> Option<(String, String)> {
    match ty {
        "commandExecution" => Some(command_summary(item)),
        "fileChange" => Some(file_change_summary(item)),
        "mcpToolCall" | "dynamicToolCall" => Some(named_tool_summary(item)),
        "webSearch" => Some(web_search_summary(item)),
        "imageView" => Some(image_view_summary(item)),
        _ => misc_tool_summary(ty, item),
    }
}

fn misc_tool_summary(ty: &str, item: &Value) -> Option<(String, String)> {
    match ty {
        "sleep" => Some(("sleep".into(), "Sleep".into())),
        "imageGeneration" => Some(("imageGeneration".into(), "Image generation".into())),
        "collabAgentToolCall" => Some(collab_summary(item)),
        _ => None,
    }
}

fn command_summary(item: &Value) -> (String, String) {
    item.get("command")
        .and_then(Value::as_str)
        .map(codex_flatten_ws)
        .filter(|s| !s.is_empty())
        .map_or_else(|| ("shell".into(), "Run".into()), labeled_bash)
}

fn labeled_bash(cmd: String) -> (String, String) {
    use crate::tool_summary::{BashToolKind, classify_bash_command};
    let inner = inner_cmd(&cmd);
    let short = shorten_middle(inner, TOOL_DISPLAY_MAX_WIDTH);
    match classify_bash_command(inner) {
        BashToolKind::Read => ("read".into(), format!("Read {short}")),
        BashToolKind::Search => ("grep".into(), format!("Search {short}")),
        BashToolKind::Edit => ("edit".into(), format!("Edit {short}")),
        BashToolKind::Run => ("shell".into(), format!("Run {short}")),
    }
}

fn inner_cmd(cmd: &str) -> &str {
    cmd.strip_prefix("/bin/bash -lc ")
        .or_else(|| cmd.strip_prefix("bash -lc "))
        .or_else(|| cmd.strip_prefix("/bin/sh -lc "))
        .or_else(|| cmd.strip_prefix("sh -lc "))
        .map_or(cmd, |rest| rest.trim_matches(|c| c == '\'' || c == '"'))
}

fn file_change_summary(item: &Value) -> (String, String) {
    let path = item
        .get("changes")
        .and_then(Value::as_array)
        .and_then(|changes| changes.first())
        .and_then(|change| change.get("path"))
        .and_then(Value::as_str)
        .map(codex_flatten_ws)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "files".into());
    (
        "edit".into(),
        format!("Edit {}", shorten_middle(&path, TOOL_DISPLAY_MAX_WIDTH)),
    )
}

fn named_tool_summary(item: &Value) -> (String, String) {
    let name = item
        .get("tool")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("tool");
    let n = name.to_ascii_lowercase();
    if n == "bash" || n == "shell" {
        let Some(cmd) = command_from_args(item.get("arguments")) else {
            return ("shell".into(), "Run".into());
        };
        return labeled_bash(cmd);
    }
    (name.into(), named_tool_label(name, item.get("arguments")))
}

fn named_tool_label(name: &str, args: Option<&Value>) -> String {
    let n = name.to_ascii_lowercase();
    if let Some(path) = path_from_args(args) {
        let short = shorten_middle(&path, TOOL_DISPLAY_MAX_WIDTH);
        if n == "read" || n.starts_with("read_") {
            return format!("Read {short}");
        }
        if n == "write" || n == "edit" || n.starts_with("write_") || n.starts_with("edit_") {
            return format!("Edit {short}");
        }
    }
    name.into()
}

fn web_search_summary(item: &Value) -> (String, String) {
    let query = item
        .get("query")
        .and_then(Value::as_str)
        .map(codex_flatten_ws)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "web".into());
    (
        "webSearch".into(),
        format!("Search {}", shorten_middle(&query, TOOL_DISPLAY_MAX_WIDTH)),
    )
}

fn image_view_summary(item: &Value) -> (String, String) {
    let path = item
        .get("path")
        .and_then(Value::as_str)
        .map(codex_flatten_ws)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "image".into());
    (
        "read".into(),
        format!("Read {}", shorten_middle(&path, TOOL_DISPLAY_MAX_WIDTH)),
    )
}

fn collab_summary(item: &Value) -> (String, String) {
    let tool = item
        .get("tool")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("collab");
    (tool.into(), format!("Collab {tool}"))
}

fn command_from_args(args: Option<&Value>) -> Option<String> {
    args.and_then(Value::as_object)
        .and_then(|a| a.get("command").or_else(|| a.get("cmd")))
        .and_then(Value::as_str)
        .map(codex_flatten_ws)
        .filter(|s| !s.is_empty())
}

fn path_from_args(args: Option<&Value>) -> Option<String> {
    args.and_then(Value::as_object)
        .and_then(|a| {
            a.get("path")
                .or_else(|| a.get("file_path"))
                .or_else(|| a.get("filePath"))
        })
        .and_then(Value::as_str)
        .map(codex_flatten_ws)
        .filter(|s| !s.is_empty())
}

fn codex_flatten_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
