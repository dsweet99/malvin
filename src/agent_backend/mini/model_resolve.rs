//! Model slug resolution for `--mini`.

pub use crate::support_paths::MINI_DEFAULT_MODEL;

/// Resolve `--model auto` to the v1 hardcoded default; pass through other slugs.
#[must_use]
pub fn resolve_mini_model(model: &str) -> String {
    if model == "auto" {
        MINI_DEFAULT_MODEL.to_string()
    } else {
        model.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mini_model_auto_returns_claude_sonnet_4() {
        assert_eq!(resolve_mini_model("auto"), MINI_DEFAULT_MODEL);
    }

    #[test]
    fn resolve_mini_model_passthrough() {
        assert_eq!(resolve_mini_model("openai/gpt-4o"), "openai/gpt-4o");
    }
}
