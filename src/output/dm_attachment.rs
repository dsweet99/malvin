use super::sentinel::{is_sentinel_line, DM_END, DM_START};

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

fn final_attachment_line(body: &str) -> Option<&str> {
    body.rsplit_once('\n')
        .map_or(body, |(_, line)| line)
        .strip_prefix("__ATTACH__: ")
        .filter(|path| !path.is_empty())
}
