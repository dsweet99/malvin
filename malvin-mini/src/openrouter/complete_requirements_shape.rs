use super::super::types::{ChatMessage, ChatRole};
use super::complete_act_inputs::new_request_text;

/// Requirements-listing turns: write string-schema JSON and pause; do not explore/fix product code.
pub(super) const REQUIREMENTS_ONLY_CUE: &str = "This New request is requirements-listing only. \
You MUST emit a ```bash fence that writes review_requirements.json with cat > to the exact \
absolute path named in the New request (never a workspace-relative shortcut unless that exact \
path is named). Use this JSON shape only: \
{\"groups\":[{\"title\":\"short label\",\"requirements\":[\"plain string\", \"another string\"]}]}. \
Each requirements array entry MUST be a plain string — never an object with id/description/name. \
Prefer the minimum number of groups; zero groups is allowed when nothing meaningful needs review \
(for example a single clear bug fix with an explicit acceptance test). \
Do not explore or edit product source. In the same bash fence, probe with python that asserts \
isinstance(req, str) for every requirement. Do not claim the file is written without that fence. \
Only after a live observation of that write may you Pause fence-less.";

/// Local retry when a requirements-listing reply still embeds object-shaped requirements.
const REQUIREMENTS_SCHEMA_NUDGE: &str = "Null Study: requirements entries were objects. \
Rewrite review_requirements.json so every requirements array item is a plain string, e.g. \
\"requirements\": [\"fix the wrapping bug\", \"keep tests unchanged\"]. \
No id/description objects. Emit a ```bash fence with cat > to the named absolute path, \
probe isinstance(req, str), then Pause only after the observation.";

/// Local retry when the JSON was written somewhere other than the named absolute path.
const REQUIREMENTS_PATH_NUDGE_PREFIX: &str =
    "Null Study: review_requirements.json was not written via ```bash to the named absolute path. \
Do not claim success in prose. Emit a bash fence like:\n```bash\ncat > ";

const REQUIREMENTS_PATH_NUDGE_SUFFIX: &str = " << 'EOF'\n\
{\"groups\":[]}\n\
EOF\n\
python3 -c \"import json; p=r'''PATH'''; d=json.load(open(p)); assert all(isinstance(r,str) for g in d['groups'] for r in g['requirements'])\"\n\
```\nReplace PATH with that same absolute path. Wait for the observation, then Pause.";

const REQUIREMENTS_MISSING_WRITE_NUDGE: &str = "Null Study: review_requirements.json was not \
written via bash to the named absolute path (under the run's .malvin_home/logs/…/ \
review_requirements.json). Emit a ```bash fence that uses cat > to that exact absolute path, \
probe isinstance(req, str), then Pause only after the observation. Do not claim the file is \
written without that fence.";

fn request_text_is_requirements_listing(t: &str) -> bool {
    t.contains("review_requirements")
        && (t.contains("Do not start implementing")
            || t.contains("Do **not** start implementing")
            || t.contains("output nothing else of substance")
            || t.contains("Write **only** the JSON")
            || t.contains("Write only the JSON")
            || t.contains("requirements-listing only"))
}

pub(super) fn new_request_is_requirements_only(messages: &[ChatMessage]) -> bool {
    new_request_text(messages).is_some_and(request_text_is_requirements_listing)
}

/// True when this completion wire is still a requirements-listing turn, even if the
/// latest New request is a bash observation that replaced the original prompt text.
pub(super) fn session_is_requirements_listing(messages: &[ChatMessage]) -> bool {
    if new_request_is_requirements_only(messages)
        || messages
            .iter()
            .any(|m| request_text_is_requirements_listing(&m.content))
        || messages
            .iter()
            .any(|m| m.content.contains("requirements-listing only"))
    {
        return true;
    }
    use super::complete_act_inputs::previous_response_text;
    match (new_request_text(messages), previous_response_text(messages)) {
        (Some(new_req), Some(prev))
            if new_req.contains("Exit code ")
                && prev.to_ascii_lowercase().contains("review_requirements") =>
        {
            let wrote_abs = (prev.contains("cat > /") || prev.contains("tee /"))
                && prev.contains("review_requirements.json");
            !wrote_abs
        }
        _ => false,
    }
}

/// Absolute requirements path from New request or any assembled message.
pub(super) fn expected_path_from_messages(messages: &[ChatMessage]) -> Option<String> {
    if let Some(p) = new_request_text(messages).and_then(expected_review_requirements_path) {
        return Some(p.to_owned());
    }
    messages.iter().rev().find_map(|m| {
        let p = expected_review_requirements_path(&m.content)?;
        if p.starts_with("/app/") {
            return None;
        }
        Some(p.to_owned())
    })
}

/// When listing still lacks an abs-path bash write, synthesize a fence the mini loop can run.
pub(super) fn force_requirements_abs_write_response(
    messages: &[ChatMessage],
    content: &str,
) -> Option<String> {
    if !session_is_requirements_listing(messages) {
        return None;
    }
    let path = expected_path_from_messages(messages)?;
    if !requirements_path_needs_retry(content, Some(path.as_str())) {
        return None;
    }
    let json = concat!(
        "{\"groups\":[{\"title\":\"Request\",\"requirements\":[",
        "\"Satisfy the user request as stated in the plan.\",",
        "\"Keep edits within the constraints named in the plan.\"",
        "]}]}"
    );
    let body = format!(
        "```bash\ncat > {path} << 'EOF'\n{json}\nEOF\n\
python3 -c \"import json; d=json.load(open(r'{path}')); \
assert all(isinstance(r, str) for g in d.get('groups', []) for r in g.get('requirements', []))\"\n\
```"
    );
    Some(super::super::memory_format::format_wire_turn(
        "- requirements: abs-path write synthesized after missing bash fence",
        &body,
    ))
}

/// True when assistant text still shows object-shaped `requirements` entries.
pub(super) fn response_has_object_shaped_requirements(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    if !lower.contains("\"requirements\"") {
        return false;
    }
    let array_opens_object = [
        "\"requirements\":[{",
        "\"requirements\": [{",
        "\"requirements\":[\n{",
        "\"requirements\": [\n{",
        "\"requirements\":[\r\n{",
        "\"requirements\": [\r\n{",
    ];
    if array_opens_object.iter().any(|p| lower.contains(p)) {
        return true;
    }
    // Object fields commonly used instead of plain strings.
    lower.contains("\"requirements\"")
        && (lower.contains("\"id\":") || lower.contains("\"description\":"))
        && lower.contains('{')
}

pub(super) fn inject_requirements_schema_nudge(messages: &mut Vec<ChatMessage>) -> bool {
    if messages.iter().any(|m| {
        matches!(m.role, ChatRole::User)
            && m.content.contains("requirements entries were objects")
    }) {
        return false;
    }
    messages.push(ChatMessage {
        role: ChatRole::User,
        content: REQUIREMENTS_SCHEMA_NUDGE.to_string(),
    });
    true
}

/// Absolute `…/review_requirements.json` path named in the New request, if any.
pub(super) fn expected_review_requirements_path(new_request: &str) -> Option<&str> {
    for token in new_request.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| {
            matches!(c, '`' | '"' | '\'' | ',' | '.' | ';' | ':' | ')' | '(' | ']' | '[')
        });
        if cleaned.ends_with("review_requirements.json") && cleaned.starts_with('/') {
            return Some(cleaned);
        }
    }
    // Fallback: scan for `/…/review_requirements.json` substrings (markdown paths).
    let needle = "review_requirements.json";
    let mut search = new_request;
    while let Some(idx) = search.find(needle) {
        let abs_end = idx + needle.len();
        let before = &search[..idx];
        if let Some(slash) = before.rfind('/') {
            // Walk left to start of path.
            let mut start = slash;
            while start > 0 {
                let ch = before.as_bytes()[start - 1] as char;
                if ch.is_whitespace() || matches!(ch, '`' | '"' | '\'' | '(' | '[') {
                    break;
                }
                start -= 1;
            }
            if before.as_bytes().get(start) == Some(&b'/') {
                let path = &search[start..abs_end];
                if path.starts_with('/') {
                    return Some(path);
                }
            }
        }
        search = &search[abs_end..];
    }
    None
}

/// `expected` is the absolute path when known; `None` still retries until an abs write appears.
pub(super) fn requirements_path_needs_retry(content: &str, expected: Option<&str>) -> bool {
    !content_has_abs_requirements_write(content, expected)
}

/// True when assistant content shows a bash (or equivalent) write to the expected path.
pub(super) fn content_has_abs_requirements_write(content: &str, expected: Option<&str>) -> bool {
    let has_fence = content.contains("```bash") || content.contains("```sh");
    match expected {
        Some(path) => {
            let mentions_path = content.contains(path);
            let write_verb = content.contains("cat >")
                || content.contains("tee ")
                || content.contains("write_text")
                || content.contains(".write(");
            has_fence && mentions_path && write_verb
        }
        None => {
            has_fence
                && (content.contains("cat > /") || content.contains("tee /"))
                && content.contains("review_requirements.json")
        }
    }
}

pub(super) fn inject_requirements_path_nudge(
    messages: &mut Vec<ChatMessage>,
    expected: Option<&str>,
) -> bool {
    messages.retain(|m| {
        !(matches!(m.role, ChatRole::User)
            && (m.content.contains("was not written to the named absolute path")
                || m.content.contains("was not written via bash to the named absolute path")
                || m.content.contains("was not written via ```bash to the named absolute path")))
    });
    let content = match expected {
        Some(path) => {
            let suffix = REQUIREMENTS_PATH_NUDGE_SUFFIX.replace("PATH", path);
            format!("{REQUIREMENTS_PATH_NUDGE_PREFIX}`{path}`{suffix}")
        }
        None => REQUIREMENTS_MISSING_WRITE_NUDGE.to_string(),
    };
    messages.push(ChatMessage {
        role: ChatRole::User,
        content,
    });
    true
}
