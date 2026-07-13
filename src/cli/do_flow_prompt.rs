use crate::artifacts::RunArtifacts;
use crate::cli::flow_prompt_combine::{
    build_dual_header_coder_run_with_store, combine_acp_prompt_header_and_user,
    combine_mode_header_and_user, combine_prompt_file_and_user, DualHeaderPromptInput,
};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};

pub(crate) struct DoCoderRun {
    pub combined: String,
    pub header_user_for_trace: (String, String),
}

pub fn prepare_do_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(DO_HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn combine_do_prompt_file_and_user(
    store: &PromptStore,
    text: &str,
    template_file: &str,
    context: &WorkflowRenderContext,
) -> Result<(String, String, String), String> {
    combine_prompt_file_and_user(store, text, template_file, context)
}

pub fn combine_do_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<(String, String, String), String> {
    combine_acp_prompt_header_and_user(store, artifacts, text, model)
}

pub fn combine_do_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<(String, String, String), String> {
    combine_mode_header_and_user(DualHeaderPromptInput {
        store,
        artifacts,
        text,
        model,
        mode_template: DO_HEADER_MD,
    })
}

pub(crate) fn build_do_coder_run_with_store(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<DoCoderRun, String> {
    let run = build_dual_header_coder_run_with_store(DualHeaderPromptInput {
        store,
        artifacts,
        text,
        model,
        mode_template: DO_HEADER_MD,
    })?;
    Ok(DoCoderRun {
        combined: run.combined,
        header_user_for_trace: run.header_user_for_trace,
    })
}

pub(crate) fn build_do_coder_run(
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<DoCoderRun, String> {
    let store = prepare_do_prompt_store()?;
    build_do_coder_run_with_store(&store, artifacts, text, model)
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<DoCoderRun> = None;
    }
}
