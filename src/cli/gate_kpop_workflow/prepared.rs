use std::collections::HashMap;
use crate::artifacts::{MalvinChecksBackup, RunArtifacts};
use crate::prompts::PromptStore;

pub(crate) struct GateKpopPrepared {
    pub artifacts: RunArtifacts,
    pub context: HashMap<String, String>,
    pub request_text: String,
    pub startup_emit_request: String,
    pub store: PromptStore,
    pub malvin_checks_backup: MalvinChecksBackup,
}

impl GateKpopPrepared {
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
    fn gate_kpop_prepared_accessors_are_covered() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let prepared = GateKpopPrepared {
            artifacts,
            context: HashMap::new(),
            request_text: "req".into(),
            startup_emit_request: "startup".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        };
        assert_eq!(prepared.request_text(), "req");
        assert_eq!(prepared.startup_emit_request, "startup");
        assert!(prepared.artifacts().run_dir.exists());
        assert!(prepared.context().is_empty());
        assert!(prepared.store().validate_exists("kpop_program_creative.md").is_ok());
        let GateKpopPrepared {
            artifacts,
            context,
            request_text,
            startup_emit_request,
            store,
            malvin_checks_backup,
        } = prepared;
        assert_eq!(request_text, "req");
        assert_eq!(startup_emit_request, "startup");
        assert!(context.is_empty());
        assert!(artifacts.run_dir.exists());
        assert!(store.validate_exists("kpop_program_creative.md").is_ok());
        assert!(matches!(
            malvin_checks_backup,
            crate::artifacts::MalvinChecksBackup::Missing
        ));
    }
}
#[cfg(test)]
#[path = "prepared_test.rs"]
mod prepared_test;#[cfg(test)]
#[path = "prepared_kiss_cov_test.rs"]
mod prepared_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<GateKpopPrepared> = None;
    }
}
