use std::collections::HashMap;
use crate::artifacts::{MalvinChecksBackup, RunArtifacts};
use crate::prompts::PromptStore;

pub(crate) struct KPopEnginePrepared {
    pub artifacts: RunArtifacts,
    pub context: HashMap<String, String>,
    pub request_text: String,
    pub startup_emit_request: String,
    pub store: PromptStore,
    pub malvin_checks_backup: MalvinChecksBackup,
}

impl KPopEnginePrepared {
    pub(crate) const fn artifacts(&self) -> &RunArtifacts {
        &self.artifacts
    }

    pub(crate) const fn context(&self) -> &HashMap<String, String> {
        &self.context
    }

    pub(crate) fn request_text(&self) -> &str {
        &self.request_text
    }

    pub(crate) const fn store(&self) -> &PromptStore {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kpop_engine_prepared_accessors_are_covered() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let prepared = KPopEnginePrepared {
            artifacts,
            context: HashMap::new(),
            request_text: "req".into(),
            startup_emit_request: "startup".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        };
        assert_eq!(prepared.request_text(), "req");
        assert_eq!(prepared.startup_emit_request, "startup");
    }
}
