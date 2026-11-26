use super::sentinel::{DM_END, DM_START, is_sentinel_line};

pub(crate) struct CompletedDirectMessage {
    pub(crate) body: String,
    pub(crate) attachment: Option<String>,
}

pub(crate) fn completed_direct_messages(text: &str) -> Vec<CompletedDirectMessage> {
    let mut out = Vec::new();
    let mut inside = false;
    let mut body = String::new();
    for raw in text.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\r', '\n']);
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
