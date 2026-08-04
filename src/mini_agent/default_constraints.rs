//! Default sticky Mini system prompt (`default_prompts/mini_system_prompt.md`).

/// Compile-time embed of Mini sticky system prompt (file stays on disk for humans).
#[must_use]
pub const fn default_mini_constraints() -> &'static str {
    include_str!("../../default_prompts/mini_system_prompt.md")
}

#[cfg(test)]
mod tests {
    use super::default_mini_constraints;

    #[test]
    fn default_mini_constraints_embeds_bash_fence_and_wire_format() {
        let text = default_mini_constraints();
        assert!(text.contains("```bash"));
        assert!(text.contains("## NEW_HISTORY"));
        assert!(text.contains("## RESPONSE"));
        assert!(text.contains("## Assembly (each completion)"));
    }
}
