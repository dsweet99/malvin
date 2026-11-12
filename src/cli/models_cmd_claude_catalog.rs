use malvin::config::model_id::ModelBackend;

use super::models_cmd_catalog::{ModelCatalog, ModelListing};

pub(super) struct ClaudeCatalog;

impl ModelCatalog for ClaudeCatalog {
    fn prefix(&self) -> &'static str {
        ModelBackend::Claude.spec().prefix()
    }

    fn list_display_models(&self) -> ModelListing {
        let prefix = self.prefix();
        ModelListing::rows(
            malvin::backends::claude_sdk::models_list::list_claude_display_models()
                .into_iter()
                .map(|(id, detail)| format!("{prefix}{id}\t{detail}"))
                .collect(),
        )
    }
}
