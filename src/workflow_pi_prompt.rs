use std::path::Path;

pub const REQUEST_INLINE_MAX_BYTES: usize = 8 * 1024;

fn fence_for(text: &str) -> String {
    let mut longest = 0;
    let mut run = 0;
    for c in text.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    "`".repeat((longest + 1).max(3))
}

fn truncate_on_char_boundary(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[must_use]
pub fn format_request_inline_text(text: &str, plan_path: &Path) -> String {
    let body = text.trim_end();
    if body.trim().is_empty() {
        return String::new();
    }
    let shown = truncate_on_char_boundary(body, REQUEST_INLINE_MAX_BYTES);
    let fence = fence_for(shown);
    let rest = if shown.len() < body.len() {
        format!(
            "\nThis is only the first {REQUEST_INLINE_MAX_BYTES} bytes. Read the rest with `cat {}` via the `bash` tool.\n",
            plan_path.display()
        )
    } else {
        String::new()
    };
    format!("The requirements file contains:\n\n{fence}text\n{shown}\n{fence}\n{rest}")
}

fn is_pi(model: &str) -> bool {
    crate::model_id::parse_model_id(model).is_ok_and(|m| m.is_pi())
}

#[must_use]
pub fn format_request_inline(model: &str, plan_path: &Path) -> String {
    if !is_pi(model) {
        return String::new();
    }
    std::fs::read_to_string(plan_path)
        .map(|text| format_request_inline_text(&text, plan_path))
        .unwrap_or_default()
}

pub fn insert_pi_prompt_keys(
    context: &mut std::collections::HashMap<String, String>,
    model: &str,
    plan_path: &Path,
) {
    context.insert(
        "request_inline".to_string(),
        format_request_inline(model, plan_path),
    );
}

#[cfg(test)]
#[path = "workflow_pi_prompt_tests.rs"]
mod tests;
