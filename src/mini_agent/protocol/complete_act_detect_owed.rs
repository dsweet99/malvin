use crate::llm_transport::{ChatMessage, ChatRole};

use super::complete_act_detect::{bash_fence_bodies, body_looks_artifact_revision};

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
            ChatRole::System => {}
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

