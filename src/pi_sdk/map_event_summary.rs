use serde_json::Value;

use crate::tool_summary::{TOOL_DISPLAY_MAX_WIDTH, shorten_middle};

#[must_use]
pub(crate) fn tool_summary_from_pi(name: Option<&str>, args: Option<&Value>) -> Option<String> {
    let label = name.unwrap_or("tool").trim();
    if label.is_empty() {
        return None;
    }
    let n = label.to_ascii_lowercase();
    let args = args.and_then(Value::as_object);
    if n == "bash" || n == "shell" || n == "powershell" {
        return Some(bash_summary(args));
    }
    if n == "grep" || n == "find" || n == "search" {
        return Some(search_summary(args));
    }
    if n == "ls" {
        return Some(ls_summary(args));
    }
    if let Some(path) = path_arg(args)
        && let Some(summary) = path_tool_summary(&n, &path)
    {
        return Some(summary);
    }
    Some(fallback_summary(label, &n, args))
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

fn search_summary(args: Option<&serde_json::Map<String, Value>>) -> String {
    let query = args
        .and_then(|a| {
            a.get("pattern")
                .or_else(|| a.get("query"))
                .or_else(|| a.get("glob"))
                .or_else(|| a.get("path"))
        })
        .and_then(Value::as_str)
        .map(flatten_ws)
        .filter(|s| !s.is_empty());
    query.map_or_else(
        || "Search".into(),
        |q| format!("Search {}", shorten_middle(&q, TOOL_DISPLAY_MAX_WIDTH)),
    )
}

fn ls_summary(args: Option<&serde_json::Map<String, Value>>) -> String {
    path_arg(args).map_or_else(
        || "List".into(),
        |p| format!("List {}", shorten_middle(&p, TOOL_DISPLAY_MAX_WIDTH)),
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

fn fallback_summary(
    label: &str,
    n: &str,
    args: Option<&serde_json::Map<String, Value>>,
) -> String {
    let title = titled_tool_name(n).unwrap_or_else(|| title_case_label(label));
    match primary_arg_snippet(args) {
        Some(snippet) => format!("{title} {snippet}"),
        None => title,
    }
}

fn titled_tool_name(n: &str) -> Option<String> {
    if n == "read" || n.starts_with("read_") {
        return Some("Read".into());
    }
    if n == "write" || n == "edit" || n.starts_with("write_") || n.starts_with("edit_") {
        return Some("Edit".into());
    }
    None
}

fn title_case_label(label: &str) -> String {
    let mut chars = label.chars();
    chars
        .next()
        .map_or_else(String::new, |first| first.to_uppercase().collect::<String>() + chars.as_str())
}

fn primary_arg_snippet(args: Option<&serde_json::Map<String, Value>>) -> Option<String> {
    let map = args.filter(|m| !m.is_empty())?;
    if let Some(path) = path_arg(Some(map)) {
        return Some(shorten_middle(&path, TOOL_DISPLAY_MAX_WIDTH));
    }
    for key in ["command", "cmd", "pattern", "query", "glob", "url", "text"] {
        if let Some(v) = map
            .get(key)
            .and_then(Value::as_str)
            .map(flatten_ws)
            .filter(|s| !s.is_empty())
        {
            return Some(shorten_middle(&v, TOOL_DISPLAY_MAX_WIDTH));
        }
    }
    let compact = serde_json::to_string(&Value::Object(map.clone())).ok()?;
    Some(shorten_middle(&compact, TOOL_DISPLAY_MAX_WIDTH))
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

#[cfg(test)]
mod tool_summary_from_pi_tests {
    use super::tool_summary_from_pi;
    use serde_json::json;

    #[test]
    fn read_includes_path_argument() {
        let summary = tool_summary_from_pi(Some("read"), Some(&json!({"path": "src/lib.rs"})));
        assert_eq!(summary.as_deref(), Some("Read src/lib.rs"));
    }

    #[test]
    fn read_without_args_is_titled_not_read_tool() {
        let summary = tool_summary_from_pi(Some("read"), None);
        assert_eq!(summary.as_deref(), Some("Read"));
        assert!(!summary.unwrap().contains("tool"));
    }

    #[test]
    fn bash_includes_command_argument() {
        let summary = tool_summary_from_pi(Some("bash"), Some(&json!({"command": "ls -la"})));
        assert_eq!(summary.as_deref(), Some("Run ls -la"));
    }

    #[test]
    fn grep_includes_pattern_argument() {
        let summary = tool_summary_from_pi(Some("grep"), Some(&json!({"pattern": "tool_call"})));
        assert_eq!(summary.as_deref(), Some("Search tool_call"));
    }

    #[test]
    fn write_maps_to_edit_with_path() {
        let summary = tool_summary_from_pi(Some("write"), Some(&json!({"path": "a.rs"})));
        assert_eq!(summary.as_deref(), Some("Edit a.rs"));
    }
}
