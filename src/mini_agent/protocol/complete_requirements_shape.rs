//! Requirements-listing cues, session detection, force-write, and nudge injectors.

use crate::llm_transport::{ChatMessage, ChatRole};
use super::complete_act_inputs::new_request_text;
use super::complete_requirements_path::{
    expected_review_requirements_path, requirements_path_needs_retry,
};

/// Requirements-listing turns: write string-schema JSON and pause; do not explore/fix product code.
pub(crate) const REQUIREMENTS_ONLY_CUE: &str = "This New request is requirements-listing only. \
You MUST emit a ```bash fence that writes review_requirements.json with cat > to the exact \
absolute path named in the New request (never a workspace-relative shortcut unless that exact \
path is named; never /app/review_requirements.json unless that exact absolute path is named). \
Use this JSON shape only: \
{\"groups\":[{\"title\":\"short label\",\"requirements\":[\"plain string\", \"another string\"]}]}. \
Each requirements array entry MUST be a plain string — never an object with id/description/name. \
Prefer one group when that covers the hard constraints; use at most two unless the request \
has clearly separable constraint clusters. Zero groups is only for requests that already \
need no work. \
Every CLI flag and positional argument named in the plan must appear in some requirement string \
(do not omit file-path args by assuming stdin-only). \
Do not explore or edit product source. In the same bash fence, probe with python that asserts \
isinstance(req, str) for every requirement. Do not claim the file is written without that fence. \
Only after a live observation of that write may you Pause fence-less — do not rewrite the file \
to any other path after a successful abs-path observation.";

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
{\"groups\":[{\"title\":\"short label\",\"requirements\":[\"plain string\", \"another string\"]}]}\n\
EOF\n\
python3 -c \"import json; p=r'''PATH'''; d=json.load(open(p)); assert all(isinstance(r,str) for g in d['groups'] for r in g['requirements']); assert d['groups'], 'groups must not be empty for this task'\"\n\
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

pub(crate) fn new_request_is_requirements_only(messages: &[ChatMessage]) -> bool {
    new_request_text(messages).is_some_and(request_text_is_requirements_listing)
}

/// True when this completion wire is still a requirements-listing turn, even if the
/// latest New request is a bash observation that replaced the original prompt text.
pub(crate) fn session_is_requirements_listing(messages: &[ChatMessage]) -> bool {
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
pub(crate) fn expected_path_from_messages(messages: &[ChatMessage]) -> Option<String> {
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
pub(crate) fn force_requirements_abs_write_response(
    messages: &[ChatMessage],
    content: &str,
) -> Option<String> {
    if !session_is_requirements_listing(messages) {
        return None;
    }
    let path = expected_path_from_messages(messages)?;
    if super::complete_requirements_path::requirements_file_on_disk_is_valid(&path) {
        return None;
    }
    if !requirements_path_needs_retry(content, Some(path.as_str())) {
        return None;
    }
    let json = concat!(
        "{\"groups\":[{\"title\":\"Plan acceptance\",\"requirements\":[",
        "\"Match the plan's documented CLI exactly: every flag and every positional path ",
        "(e.g. file args — do not substitute stdin-only).\",",
        "\"Match the plan's documented stdout/stderr and exit-code contracts exactly.\",",
        "\"Keep edits within the workspace constraints named in the plan.\"",
        "]}]}"
    );
    let body = format!(
        "```bash\ncat > {path} << 'EOF'\n{json}\nEOF\n\
python3 -c \"import json; d=json.load(open(r'{path}')); \
assert all(isinstance(r, str) for g in d.get('groups', []) for r in g.get('requirements', []))\"\n\
```"
    );
    Some(crate::mini_agent::protocol::memory_format::format_wire_turn(
        "- requirements: abs-path write synthesized after missing bash fence",
        &body,
    ))
}

pub(crate) fn inject_requirements_schema_nudge(messages: &mut Vec<ChatMessage>) -> bool {
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

pub(crate) fn inject_requirements_path_nudge(
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
