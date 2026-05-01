//! `{{ key }}` / `$key` expansion for prompt files.
#![allow(clippy::implicit_hasher)]

use std::collections::HashMap;

use super::PromptError;
use super::PromptStore;

pub fn merge_header_and_coding_rules(header_expanded: &str, rules_expanded: &str) -> String {
    let h = header_expanded.trim();
    let r = rules_expanded.trim();
    match (h.is_empty(), r.is_empty()) {
        (true, true) => String::new(),
        (true, false) => r.to_string(),
        (false, true) => h.to_string(),
        (false, false) => format!("{h}\n\n{r}"),
    }
}

pub fn render_template(prompt_text: &str, context: &HashMap<String, String>) -> String {
    let mut keys: Vec<&String> = context.keys().collect();
    keys.sort_unstable();
    let mut translated = prompt_text.to_string();
    for key in keys {
        let needle = format!("{{{{ {key} }}}}");
        let dollar = format!("${key}");
        translated = translated.replace(&needle, &dollar);
    }
    substitute_template(&translated, context)
}

/// `$identifier` replacement similar to `string.Template.safe_substitute` (no `${}` brace forms).
pub fn substitute_template(template: &str, context: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let chars: Vec<char> = template.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            if chars[i + 1] == '$' {
                out.push('$');
                i += 2;
                continue;
            }
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            if end > start {
                let key: String = chars[start..end].iter().collect();
                if let Some(val) = context.get(&key) {
                    out.push_str(val);
                    i = end;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn render_mbc2_for_scheduled_kpop_block(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, PromptError> {
    let mut ctx = context.clone();
    ctx.insert("coding_rules".to_string(), String::new());
    store.render_prompt_only("mbc2.md", &ctx)
}

pub fn merged_coding_rules(
    store: &PromptStore,
    context: &HashMap<String, String>,
) -> Result<String, PromptError> {
    let mut render_context: HashMap<String, String> = context.clone();
    render_context
        .entry("memories".to_string())
        .or_default();
    let header_raw = store.load_header();
    let header_expanded = render_template(&header_raw, &render_context);
    let rules_raw = store.load_coding_rules();
    let rules_expanded = render_template(&rules_raw, &render_context);
    let merged = merge_header_and_coding_rules(&header_expanded, &rules_expanded);
    super::enforce_no_unresolved_braces(&merged)?;
    Ok(merged)
}

#[cfg(test)]
mod template_kiss {
    #[test]
    fn kiss_stringify_template() {
        let _ = stringify!(super::render_mbc2_for_scheduled_kpop_block);
        let _ = stringify!(super::merged_coding_rules);
    }

    #[test]
    fn substitute_template_treats_double_dollar_as_literal() {
        let mut ctx = std::collections::HashMap::new();
        ctx.insert("plan_path".to_string(), "/tmp/plan".to_string());
        assert_eq!(
            super::substitute_template("use $$plan_path", &ctx),
            "use $plan_path"
        );
    }
}
