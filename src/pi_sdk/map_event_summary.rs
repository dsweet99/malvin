use serde_json::Value;

use crate::tool_summary::{TOOL_DISPLAY_MAX_WIDTH, shorten_middle};
pub(super) fn tool_summary_from_pi(name: Option<&str>, args: Option<&Value>) -> Option<String> {
    let label = name.unwrap_or("tool").trim();
    if label.is_empty() {
        return None;
    }
    let n = label.to_ascii_lowercase();
    let args = args.and_then(Value::as_object);
    if n == "bash" || n == "shell" {
        return Some(bash_summary(args));
    }
    if let Some(path) = path_arg(args)
        && let Some(summary) = path_tool_summary(&n, &path) {
            return Some(summary);
        }
    Some(label.to_string())
}

fn bash_summary(args: Option<&serde_json::Map<String, Value>>) -> String {
    let cmd = args
        .and_then(|a| a.get("command").or_else(|| a.get("cmd")))
        .and_then(Value::as_str)
        .map(flatten_ws)
        .filter(|s| !s.is_empty());
    cmd.map_or_else(
        || "Run".into(),
        |c| format!("Run {}", shorten_middle(&c, TOOL_DISPLAY_MAX_WIDTH)),
    )
}

fn path_arg(args: Option<&serde_json::Map<String, Value>>) -> Option<String> {
    args.and_then(|a| {
        a.get("path")
            .or_else(|| a.get("file_path"))
            .or_else(|| a.get("filePath"))
    })
    .and_then(Value::as_str)
    .map(flatten_ws)
    .filter(|s| !s.is_empty())
}

fn path_tool_summary(n: &str, path: &str) -> Option<String> {
    let short = shorten_middle(path, TOOL_DISPLAY_MAX_WIDTH);
    if n == "read" || n.starts_with("read_") {
        return Some(format!("Read {short}"));
    }
    if n == "write" || n == "edit" || n.starts_with("write_") || n.starts_with("edit_") {
        return Some(format!("Edit {short}"));
    }
    None
}

pub(super) fn flatten_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod flatten_ws_tests {
    use super::flatten_ws;

    #[test]
    fn flatten_ws_collapses_runs_of_whitespace() {
        assert_eq!(flatten_ws("echo   \t a\t\tb\n c"), "echo a b c");
    }

    #[test]
    fn flatten_ws_trims_and_handles_degenerate_inputs() {
        assert_eq!(flatten_ws("  padded  "), "padded");
        assert_eq!(flatten_ws("token"), "token");
        assert_eq!(flatten_ws(""), "");
        assert_eq!(flatten_ws(" \t\n "), "");
    }
}
