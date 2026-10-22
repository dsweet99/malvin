use malvin::config::model_id::ModelBackend;

use super::models_cmd_catalog::{ModelCatalog, ModelListing};

pub(super) struct ClaudeCatalog;

impl ModelCatalog for ClaudeCatalog {
    fn prefix(&self) -> &'static str {
        ModelBackend::Claude.spec().prefix()
    }

    fn list_display_models(&self) -> ModelListing {
        ModelListing::rows(vec![
            "claude:sonnet\tClaude Sonnet".into(),
            "claude:opus\tClaude Opus".into(),
            "claude:haiku\tClaude Haiku".into(),
        ])
    }
}
