use crate::openrouter::types::{ChatMessage, ChatRole};

pub(super) use super::complete_act_inputs::{
    latest_observation_has_nonzero_exit, previous_response_text,
};

pub(super) fn response_has_act_fence(content: &str) -> bool {
    content.contains("```bash") || content.contains("``` bash")
}

pub(super) fn bash_fence_bodies<'a>(
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

pub(super) fn body_looks_artifact_revision(body: &str) -> bool {
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

pub(super) fn history_has_exterior_without_artifact_act(
    messages: &[ChatMessage],
    pending: Option<&str>,
) -> bool {
    // Prefer Previous response (+ pending draft); fall back to Assistant scan for legacy lists.
    let mut bodies = Vec::new();
    if let Some(prev) = previous_response_text(messages) {
        push_bash_bodies(prev, &mut bodies);
    } else {
        bodies = bash_fence_bodies(messages, None);
    }
    if let Some(content) = pending {
        push_bash_bodies(content, &mut bodies);
    }
    bodies.iter().any(|b| body_looks_exterior(b))
        && !bodies.iter().any(|b| body_looks_artifact_revision(b))
}

pub(super) fn history_has_any_artifact_act(messages: &[ChatMessage]) -> bool {
    bash_fence_bodies(messages, None)
        .iter()
        .any(|b| body_looks_artifact_revision(b))
}

/// After a named-artifact revision, private Exit does not clear debt when the request
/// names owed runnable spans; only a later request-named-shaped Act's Exit pays.
pub(crate) fn artifact_act_lacks_following_observation(messages: &[ChatMessage]) -> bool {
    let Some(rev_i) = last_revision_index(messages) else {
        return false;
    };
    let owed = owed_spans(messages);
    if owed.is_empty() {
        return !has_exit_after(messages, rev_i);
    }
    !named_probe_exit_after(messages, rev_i, &owed)
}

fn last_revision_index(messages: &[ChatMessage]) -> Option<usize> {
    messages.iter().enumerate().rev().find_map(|(i, m)| {
        (matches!(m.role, ChatRole::Assistant)
            && bash_fence_bodies(std::slice::from_ref(m), None)
                .iter()
                .any(|b| body_looks_artifact_revision(b)))
        .then_some(i)
    })
}

fn has_exit_after(messages: &[ChatMessage], rev_i: usize) -> bool {
    messages[rev_i + 1..].iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.lines().any(|l| l.starts_with("Exit code "))
    })
}

fn named_probe_exit_after(messages: &[ChatMessage], rev_i: usize, owed: &[String]) -> bool {
    let mut pending = false;
    for m in &messages[rev_i + 1..] {
        match m.role {
            ChatRole::Assistant => {
                pending = bash_fence_bodies(std::slice::from_ref(m), None)
                    .iter()
                    .any(|b| owed.iter().any(|s| norm(b).contains(&norm(s))));
            }
            ChatRole::User => {
                if pending && m.content.lines().any(|l| l.starts_with("Exit code ")) {
                    return true;
                }
                pending = false;
            }
            _ => {}
        }
    }
    false
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn owed_spans(messages: &[ChatMessage]) -> Vec<String> {
    let Some(req) = messages.iter().find_map(|m| {
        (matches!(m.role, ChatRole::User)
            && !m.content.lines().any(|l| l.starts_with("Exit code "))
            && !m.content.contains("Emit an Act fence now")
            && m.content.len() > 40)
        .then_some(m.content.as_str())
    }) else {
        return Vec::new();
    };
    let mut spans = Vec::new();
    let mut rest = req;
    while let Some(start) = rest.find('`') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('`') else { break };
        push_span(&mut spans, after[..end].trim());
        rest = &after[end + 1..];
    }
    for line in req.lines() {
        push_span(&mut spans, line.trim());
    }
    spans
}

fn push_span(spans: &mut Vec<String>, s: &str) {
    if !looks_runnable(s) {
        return;
    }
    let n = norm(s);
    if spans.iter().any(|e| norm(e) == n) {
        return;
    }
    spans.push(s.to_string());
}

fn looks_runnable(s: &str) -> bool {
    if s.is_empty() || s.len() > 200 || s.contains('\n') || s.starts_with("http://") || s.starts_with("https://")
    {
        return false;
    }
    let first = s.split_whitespace().next().unwrap_or("");
    !first.is_empty()
        && first
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
        && (s.split_whitespace().count() >= 2 || s.contains(" -") || s.contains(" --") || first.contains('/'))
}

