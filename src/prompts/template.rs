// `{{ key }}` / `$key` expansion for prompt files.

use std::collections::HashMap;

#[must_use]
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

#[allow(clippy::implicit_hasher)]
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
#[must_use]
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

#[cfg(test)]
mod template_kiss {
    #[test]
    fn kiss_stringify_template() {
        let _ = stringify!(crate::prompts::render_mbc2_for_scheduled_kpop_block);
        let _ = stringify!(crate::prompts::merged_coding_rules);
    }

    #[test]
    fn render_template_replaces_brace_and_dollar_keys() {
        let mut ctx = std::collections::HashMap::new();
        ctx.insert("name".to_string(), "world".to_string());
        let out = super::render_template("Hello {{ name }}", &ctx);
        assert_eq!(out, "Hello world");
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
