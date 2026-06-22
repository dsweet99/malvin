#![allow(unused_imports, dead_code)]

use crate::acp::{AgentClient, AgentError};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::PromptStore;
use crate::run_timing::{self, RunTiming};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

mod helpers;

pub use helpers::{
    check_abort, clear_review_file, fail_on_abort_for_artifacts, format_exp_log_relative,
    format_prompt_path, workflow_context, workflow_context_paths_only,
};
pub(crate) use helpers::{insert_formatted, prompt_md_stem};
mod workflow_merge;

pub use workflow_merge::merge_string_run_and_restore;

pub(crate) mod orchestrator_test_support;

mod orchestrator_kiss_coverage;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct WorkflowError(pub String);

pub(crate) fn prefer_primary_errors_over_timing(
    workflow_result: Result<(), WorkflowError>,
    end_result: Result<(), WorkflowError>,
    timing_result: Result<(), WorkflowError>,
) -> Result<(), WorkflowError> {
    if matches!((&workflow_result, &end_result), (Ok(()), Ok(()))) {
        timing_result
    } else {
        let primary = match (workflow_result, end_result) {
            (Err(w), Err(e)) => Err(WorkflowError(format!("{}; end: {}", w.0, e.0))),
            (Err(w), Ok(())) => Err(w),
            (Ok(()), Err(e)) => Err(e),
            (Ok(()), Ok(())) => Ok(()),
        };
        match (primary, timing_result) {
            (Err(p), Err(WorkflowError(timing))) => {
                Err(WorkflowError(format!("{}; timing: {timing}", p.0)))
            }
            (r, _) => r,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub max_loops: usize,
}

pub struct Orchestrator<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub config: WorkflowConfig,
    pub progress_callback: Box<dyn FnMut(&str) + Send + 'a>,
    pub session_dotfile_backups: SessionDotfileBackups,
}

impl Orchestrator<'_> {
    pub(super) fn attach_run_timing(&mut self) -> Arc<Mutex<RunTiming>> {
        self.client.attach_run_timing_for_session()
    }

    pub(super) fn emit_run_timing_artifact(
        &mut self,
        timing: &Arc<Mutex<RunTiming>>,
    ) -> Result<(), WorkflowError> {
        let res = run_timing::finalize_and_emit_run_timing(&self.artifacts.run_dir, timing);
        self.client.timing = None;
        res.map_err(|e| WorkflowError(format!("run timing: {e}")))
    }

    pub(super) fn fail_on_abort_result(&self) -> Result<(), WorkflowError> {
        fail_on_abort_for_artifacts(self.artifacts)
    }
}

#[cfg(test)]
mod orchestrator_smoke_tests {
    use super::*;
    use crate::orchestrator::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::prompts::PromptStore;

    #[test]
    fn workflow_config_and_orchestrator_smoke_fields() {
        let cfg = WorkflowConfig { max_loops: 3 };
        assert_eq!(cfg.max_loops, 3);
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, store, _ctx) = workflow_ctx_for_smoke(&tmp, "orch_smoke");
        let mut client = no_session_client();
        let mut orch = Orchestrator {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            config: WorkflowConfig { max_loops: 1 },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: empty_dotfile_backups(),
        };
        let timing = orch.attach_run_timing();
        orch.fail_on_abort_result().expect("no abort");
        orch.emit_run_timing_artifact(&timing)
            .expect("timing artifact");
    }
}
#[cfg(test)]
#[path = "mod_kiss_cov_test.rs"]
mod mod_kiss_cov_test;
#[cfg(test)]
#[path = "mod_test.rs"]
mod mod_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<Orchestrator> = None;
        let _: Option<WorkflowConfig> = None;
    }
}
