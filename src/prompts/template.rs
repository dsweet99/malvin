//! `{{ key }}` / `$key` expansion for prompt files.

use std::collections::HashMap;

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
    let mut translated = prompt_text.to_string();
    for key in context.keys() {
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
