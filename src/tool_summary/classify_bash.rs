//! Heuristic bash command classifier for mini tool summaries.

use std::time::Duration;

use super::{
    escape_tool_subject_fragment, humanize_duration, shorten_middle, TOOL_DISPLAY_MAX_WIDTH,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BashToolKind {
    Read,
    Search,
    Edit,
    Run,
}

pub fn classify_bash_command(cmd: &str) -> BashToolKind {
    let trimmed = cmd.trim();
    if trimmed.contains("sed -i") || trimmed.contains(">>") || trimmed.contains(" tee ") {
        return BashToolKind::Edit;
    }
    let first = trimmed.split_whitespace().next().unwrap_or("");
    match first {
        "cat" | "head" | "tail" => BashToolKind::Read,
        "rg" | "grep" | "find" => BashToolKind::Search,
        _ if trimmed.starts_with("sed -n") => BashToolKind::Read,
        _ => BashToolKind::Run,
    }
}

fn extract_read_subject(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.first() == Some(&"sed") {
        return parts.last().copied().unwrap_or("file").to_string();
    }
    parts.get(1).copied().unwrap_or("file").to_string()
}

fn extract_search_subject(cmd: &str) -> String {
    let trimmed = cmd.trim();
    if trimmed.starts_with("find ") {
        return "files".to_string();
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    parts.get(1).copied().unwrap_or("pattern").to_string()
}

fn extract_edit_subject(cmd: &str) -> String {
    if let Some(after) = cmd.split(">>").nth(1) {
        let path = after.split_whitespace().next().unwrap_or("file");
        return path.to_string();
    }
    cmd.split_whitespace().last().unwrap_or("file").to_string()
}

pub fn format_classified_tool_line(
    kind: BashToolKind,
    command: &str,
    exit_code: i32,
    elapsed: Duration,
) -> String {
    let dur = humanize_duration(elapsed);
    let subject = match kind {
        BashToolKind::Read => {
            let path = extract_read_subject(command);
            shorten_middle(
                &escape_tool_subject_fragment(&path),
                TOOL_DISPLAY_MAX_WIDTH,
            )
        }
        BashToolKind::Search => {
            let q = extract_search_subject(command);
            shorten_middle(
                &escape_tool_subject_fragment(&q),
                TOOL_DISPLAY_MAX_WIDTH,
            )
        }
        BashToolKind::Edit => {
            let path = extract_edit_subject(command);
            shorten_middle(
                &escape_tool_subject_fragment(&path),
                TOOL_DISPLAY_MAX_WIDTH,
            )
        }
        BashToolKind::Run => {
            let flattened = escape_tool_subject_fragment(command.trim());
            shorten_middle(&flattened, TOOL_DISPLAY_MAX_WIDTH)
        }
    };
    let prefix = match kind {
        BashToolKind::Read => "Read",
        BashToolKind::Search => "Search",
        BashToolKind::Edit => "Edit",
        BashToolKind::Run => "Run",
    };
    if exit_code == 0 {
        format!("{prefix} {subject} · {dur} · ✓")
    } else {
        format!("{prefix} {subject} · {dur} · ✗ exit {exit_code}")
    }
}

pub const fn bash_kind_wire_name(kind: BashToolKind) -> &'static str {
    match kind {
        BashToolKind::Read => "read",
        BashToolKind::Search => "search",
        BashToolKind::Edit => "edit",
        BashToolKind::Run => "execute",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_read_commands() {
        assert_eq!(classify_bash_command("cat file.txt"), BashToolKind::Read);
        assert_eq!(classify_bash_command("head -n 5 foo"), BashToolKind::Read);
        assert_eq!(classify_bash_command("sed -n '1,5p' bar"), BashToolKind::Read);
    }

    #[test]
    fn classify_search_commands() {
        assert_eq!(classify_bash_command("rg pattern"), BashToolKind::Search);
        assert_eq!(classify_bash_command("grep foo *.rs"), BashToolKind::Search);
        assert_eq!(classify_bash_command("find . -name '*.rs'"), BashToolKind::Search);
    }

    #[test]
    fn classify_edit_commands() {
        assert_eq!(classify_bash_command("sed -i 's/a/b/' f"), BashToolKind::Edit);
        assert_eq!(classify_bash_command("echo x >> out.txt"), BashToolKind::Edit);
    }

    #[test]
    fn classify_pipeline_falls_back_to_run() {
        assert_eq!(
            classify_bash_command("curl https://x | jq ."),
            BashToolKind::Run
        );
    }

    #[test]
    fn format_read_line() {
        let line = format_classified_tool_line(
            BashToolKind::Read,
            "cat README.md",
            0,
            Duration::from_millis(10),
        );
        assert!(line.starts_with("Read README.md"));
        assert!(line.contains("✓"));
    }
}
