//! Model slug resolution for the malvin-mini HTTP backend (`mini:openrouter/…` / `mini:local/…`).

pub use crate::support_paths::MINI_DEFAULT_MODEL;

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
        assert_eq!(
            resolve_mini_model("mini:openrouter/auto"),
            MINI_DEFAULT_MODEL
        );
    }

    #[test]
    fn resolve_mini_model_passthrough() {
        assert_eq!(
            resolve_mini_model("mini:openrouter/openai/gpt-4o"),
            "openai/gpt-4o"
        );
        assert_eq!(
            resolve_mini_model("mini:local/qwen35_9b_q4"),
            "qwen35_9b_q4"
        );
    }
}
