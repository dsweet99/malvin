//! Absolute-path detection and RESPONSE-scoped write checks for requirements listing.

use crate::mini_agent::protocol::memory_format::{NEW_HISTORY_HEADING, RESPONSE_HEADING};

/// Absolute `…/review_requirements.json` path named in the New request, if any.
pub(crate) fn expected_review_requirements_path(new_request: &str) -> Option<&str> {
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
pub(crate) fn requirements_path_needs_retry(content: &str, expected: Option<&str>) -> bool {
    !content_has_abs_requirements_write(content, expected)
}

/// True when `path` already holds valid string-schema review requirements on disk.
///
/// After a force-write or an earlier successful fence, later fence-less Pause replies must
/// not keep path-nudging merely because the RESPONSE body omits another `cat >`.
pub(crate) fn requirements_file_on_disk_is_valid(path: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    let Some(groups) = value.get("groups").and_then(|g| g.as_array()) else {
        return false;
    };
    groups_have_nonempty_string_requirements(groups)
}

fn groups_have_nonempty_string_requirements(groups: &[serde_json::Value]) -> bool {
    for group in groups {
        let Some(reqs) = group.get("requirements").and_then(|r| r.as_array()) else {
            return false;
        };
        if reqs.is_empty() {
            return false;
        }
        for item in reqs {
            let Some(s) = item.as_str() else {
                return false;
            };
            if s.trim().is_empty() {
                return false;
            }
        }
    }
    true
}

/// Body that can carry an executable bash fence for requirements writes.
///
/// Mini classify runs on the RESPONSE section only; bash inside NEW_HISTORY is never
/// executed. Path / force checks must ignore NEW_HISTORY so a history sketch cannot
/// suppress retries while RESPONSE is still a prose claim.
fn requirements_action_body(content: &str) -> &str {
    let Some(resp_pos) = content.find(RESPONSE_HEADING) else {
        return content;
    };
    if let Some(hist_pos) = content.find(NEW_HISTORY_HEADING)
        && hist_pos > resp_pos
    {
        return content;
    }
    content[resp_pos + RESPONSE_HEADING.len()..].trim()
}

/// True when assistant content shows a bash (or equivalent) write to the expected path.
pub(crate) fn content_has_abs_requirements_write(content: &str, expected: Option<&str>) -> bool {
    let body = requirements_action_body(content);
    let has_fence = body.contains("```bash") || body.contains("```sh");
    match expected {
        Some(path) => {
            let mentions_path = body.contains(path);
            let write_verb = body.contains("cat >")
                || body.contains("tee ")
                || body.contains("write_text")
                || body.contains(".write(");
            has_fence && mentions_path && write_verb
        }
        None => {
            has_fence
                && (body.contains("cat > /") || body.contains("tee /"))
                && body.contains("review_requirements.json")
        }
    }
}

/// True when assistant text still shows object-shaped `requirements` entries.
pub(crate) fn response_has_object_shaped_requirements(content: &str) -> bool {
    let body = requirements_action_body(content);
    let lower = body.to_ascii_lowercase();
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
    lower.contains("\"requirements\"")
        && (lower.contains("\"id\":") || lower.contains("\"description\":"))
        && lower.contains('{')
}
