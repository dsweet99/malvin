//! Model slug resolution for the `OpenRouter` (malvin-mini) backend.

pub use crate::support_paths::MINI_DEFAULT_MODEL;

/// Resolve an `OpenRouter` model id to the provider slug (strips `openrouter:` when present).
#[must_use]
pub fn resolve_mini_model(model: &str) -> String {
    let slug = crate::model_id::provider_slug(model);
    if slug.is_empty() {
        MINI_DEFAULT_MODEL.to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mini_model_auto_returns_default_slug() {
        assert_eq!(resolve_mini_model("openrouter:auto"), MINI_DEFAULT_MODEL);
        assert_eq!(resolve_mini_model("auto"), "auto");
    }

    #[test]
    fn resolve_mini_model_passthrough() {
        assert_eq!(
            resolve_mini_model("openrouter:openai/gpt-4o"),
            "openai/gpt-4o"
        );
        assert_eq!(resolve_mini_model("openai/gpt-4o"), "openai/gpt-4o");
    }
}
