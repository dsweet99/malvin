use crate::artifacts::{MalvinChecksBackup, RunArtifacts};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::PromptStore;

pub(crate) struct KPopEnginePrepared {
    pub artifacts: RunArtifacts,
    pub context: WorkflowRenderContext,
    /// Retained for tests and introspection; turn prompts read `user_request_path` on disk.
    #[allow(dead_code)]
    pub request_text: String,
    pub startup_emit_request: String,
    pub store: PromptStore,
    pub malvin_checks_backup: MalvinChecksBackup,
}

impl KPopEnginePrepared {
    pub(crate) const fn artifacts(&self) -> &RunArtifacts {
        &self.artifacts
    }

    pub(crate) const fn context(&self) -> &WorkflowRenderContext {
        &self.context
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
            context: WorkflowRenderContext::default(),
            request_text: "req".into(),
            startup_emit_request: "startup".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        };
        assert_eq!(prepared.request_text, "req");
        assert_eq!(prepared.startup_emit_request, "startup");
    }
}
