use crate::mini_agent::protocol::memory_format::RESPONSE_HEADING;
use crate::llm_transport::{ChatMessage, ChatRole};

pub(crate) use super::complete_act_inputs::{
    latest_observation_has_nonzero_exit, latest_observation_has_zero_exit, previous_response_text,
};

/// Pending/current wire text scoped to `## RESPONSE` when present so fences inside
/// `## NEW_HISTORY` do not count as Act evidence.
pub(crate) fn response_section_or_raw(content: &str) -> &str {
    match content.find(RESPONSE_HEADING) {
        Some(i) => content[i + RESPONSE_HEADING.len()..].trim(),
        None => content,
    }
}

pub(crate) fn response_has_act_fence(content: &str) -> bool {
    let body = response_section_or_raw(content);
    body.contains("```bash") || body.contains("``` bash")
}

pub(crate) fn bash_fence_bodies<'a>(
    messages: &'a [ChatMessage],
    pending: Option<&'a str>,
) -> Vec<&'a str> {
    let mut bodies = Vec::new();
    for m in messages {
        if matches!(m.role, ChatRole::Assistant) {
            push_bash_bodies(&m.content, &mut bodies);
        }
    }
    if let Some(content) = pending {
        push_bash_bodies(content, &mut bodies);
    }
    bodies
}

fn push_bash_bodies<'a>(content: &'a str, out: &mut Vec<&'a str>) {
    let mut rest = content;
    while let Some(start) = rest.find("```") {
        let after_ticks = &rest[start + 3..];
        let after_lang = after_ticks.trim_start();
        let is_bash = after_lang.starts_with("bash")
            || after_lang.starts_with("sh")
            || after_lang.starts_with('\n');
        if !is_bash {
            rest = &rest[start + 3..];
            continue;
        }
        let Some(body_start) = after_ticks.find('\n').map(|i| i + 1) else {
            rest = &rest[start + 3..];
            continue;
        };
        let body = &after_ticks[body_start..];
        let end = body.find("```").unwrap_or(body.len());
        out.push(&body[..end]);
        rest = &body[end..];
        if rest.starts_with("```") {
            rest = &rest[3..];
        }
    }
}

fn body_looks_exterior(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    lower.contains("https://")
        || lower.contains("http://")
        || lower.contains("curl ")
        || lower.contains("wget ")
        || lower.contains("urllib")
}

pub(crate) fn body_looks_artifact_revision(body: &str) -> bool {
    body.contains("sed -i")
        || body.contains(">>")
        || body.contains(" tee ")
        || body.contains("tee ")
        || body.contains("cat >")
        || body.contains("cat>>")
        || has_redirect_write(body)
}

fn has_redirect_write(body: &str) -> bool {
    body.lines().any(|line| {
        let t = line.trim();
        !t.is_empty()
            && !t.starts_with('#')
            && t.contains('>')
            && !t.contains(">>")
            && !t.contains("2>")
            && !t.contains("&>")
            && !t.contains("->")
            && !t.contains(">=")
    })
}

pub(crate) fn history_has_exterior_without_artifact_act(
    messages: &[ChatMessage],
    pending: Option<&str>,
) -> bool {
    // Previous response (+ pending RESPONSE body only). No durable-transcript Assistant scan.
    let mut bodies = Vec::new();
    if let Some(prev) = previous_response_text(messages) {
        push_bash_bodies(prev, &mut bodies);
    }
    if let Some(content) = pending {
        push_bash_bodies(response_section_or_raw(content), &mut bodies);
    }
    bodies.iter().any(|b| body_looks_exterior(b))
        && !bodies.iter().any(|b| body_looks_artifact_revision(b))
}

pub(crate) fn history_has_any_artifact_act(messages: &[ChatMessage]) -> bool {
    bash_fence_bodies(messages, None)
        .iter()
        .any(|b| body_looks_artifact_revision(b))
}

/// Fence-less RESPONSE that claims a write/create without a bash Act in that same body.
pub(crate) fn response_claims_write_without_fence(content: &str) -> bool {
    if response_has_act_fence(content) {
        return false;
    }
    let body = response_section_or_raw(content).to_ascii_lowercase();
    const CLAIMS: &[&str] = &[
        "i've created",
        "i have created",
        "i've written",
        "i have written",
        "i wrote",
        "i created",
        "successfully created",
        "successfully wrote",
        "artifact written",
        "file is written",
        "script is created",
        "is created, executable",
        "created successfully",
        "written successfully",
        "wrote the script",
        "created the script",
        "created `bin/",
        "wrote `bin/",
    ];
    CLAIMS.iter().any(|c| body.contains(c))
}

/// Prose write-claim with no Act fence is unpaid unless the latest green observation
/// already followed an artifact-revision bash in the previous response.
pub(crate) fn unpaid_prose_write_claim(messages: &[ChatMessage], pending: &str) -> bool {
    if !response_claims_write_without_fence(pending) {
        return false;
    }
    if !latest_observation_has_zero_exit(messages) {
        return true;
    }
    let Some(prev) = previous_response_text(messages) else {
        return true;
    };
    let mut bodies = Vec::new();
    push_bash_bodies(response_section_or_raw(prev), &mut bodies);
    !bodies.iter().any(|b| body_looks_artifact_revision(b))
}

pub(crate) use super::complete_act_detect_owed::artifact_act_lacks_following_observation;


