use super::code_fence::CodeFence;
use super::sentinel::{DM_END, DM_START, is_sentinel_line};

pub(crate) struct CompletedDirectMessage {
    pub(crate) body: String,
    pub(crate) attachment: Option<String>,
}

pub(crate) fn completed_direct_messages(text: &str) -> Vec<CompletedDirectMessage> {
    let mut out = Vec::new();
    let mut inside = false;
    let mut fence = CodeFence::new();
    let mut body = String::new();
    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\r', '\n']);
        if !inside && (fence.observe(line) || fence.is_open()) {
            continue;
        }
        if take_marker(&mut inside, &mut body, &mut out, line) {
            continue;
        }
        if inside {
            body.push_str(raw);
        }
    }
    out
}

fn take_marker(
    inside: &mut bool,
    body: &mut String,
    out: &mut Vec<CompletedDirectMessage>,
    line: &str,
) -> bool {
    if !*inside && is_sentinel_line(line, DM_START) {
        *inside = true;
        body.clear();
        return true;
    }
    if *inside && is_sentinel_line(line, DM_END) {
        if body.ends_with('\n') {
            body.pop();
        }
        if body.ends_with('\r') {
            body.pop();
        }
        let body = std::mem::take(body);
        out.push(CompletedDirectMessage {
            attachment: final_attachment_line(&body).map(str::to_string),
            body: body_without_final_attachment_line(body),
        });
        *inside = false;
        return true;
    }
    false
}

fn body_without_final_attachment_line(mut body: String) -> String {
    if final_attachment_line(&body).is_none() {
        return body;
    }
    if let Some(last_newline) = body.rfind('\n') {
        body.truncate(last_newline);
    } else {
        body.clear();
    }
    body
}

const ATTACH_MARKER: &str = "__ATTACH__";

fn final_attachment_line(body: &str) -> Option<&str> {
    attachment_path(body.rsplit_once('\n').map_or(body, |(_, line)| line))
}

pub(crate) fn attachment_path(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix(ATTACH_MARKER)?;
    let path = rest
        .trim_start_matches(|ch: char| ch.is_whitespace() || matches!(ch, ':' | '='))
        .trim_end();
    let path = unwrap_path(path).trim();
    (!path.is_empty()).then_some(path)
}

fn unwrap_path(path: &str) -> &str {
    for (open, close) in [('`', '`'), ('"', '"'), ('\'', '\''), ('<', '>')] {
        if let Some(inner) = path
            .strip_prefix(open)
            .and_then(|rest| rest.strip_suffix(close))
        {
            return inner;
        }
    }
    path
}

pub(crate) fn existing_attachment(attachment: Option<String>) -> Option<String> {
    let path = attachment?;
    if std::path::Path::new(&path).exists() {
        return Some(path);
    }
    super::print_log_warning(&format!(
        "dm attachment not found, sending without it: {path}"
    ));
    None
}

#[cfg(test)]
mod fence_tests {
    use super::super::sentinel::{DM_END, DM_START};
    use super::completed_direct_messages;

    #[test]
    fn a_dm_quoted_inside_a_code_fence_is_not_sent() {
        let quoted = format!("Example:\n```\n{DM_START}\nhi\n{DM_END}\n```\n");
        assert!(completed_direct_messages(&quoted).is_empty());
    }

    #[test]
    fn a_dm_may_contain_a_code_fence_and_follow_one() {
        let text = format!("```\nx\n```\n{DM_START}\n```\ncode\n```\n{DM_END}\n");
        let messages = completed_direct_messages(&text);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].body, "```\ncode\n```");
    }
}
