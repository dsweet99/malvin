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

#[cfg(test)]
mod helpers_tests;

mod workflow_merge;

pub use workflow_merge::merge_string_run_and_restore;

#[cfg(test)]
pub(crate) mod orchestrator_test_support;

#[cfg(test)]
mod orchestrator_kiss_coverage;

pub mod session_flow;

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

mod coder_prompt_impl;
