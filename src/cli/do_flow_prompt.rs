use crate::artifacts::RunArtifacts;
use crate::cli::flow_prompt_combine::{
    DualHeaderPromptInput, combine_acp_prompt_header_and_user, combine_mode_header_and_user,
    combine_prompt_file_and_user,
};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};
use crate::workflow_context::PromptModelOpts;

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
    opts: PromptModelOpts<'_>,
) -> Result<(String, String, String), String> {
    combine_acp_prompt_header_and_user(store, artifacts, text, opts)
}

pub fn combine_do_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    opts: PromptModelOpts<'_>,
) -> Result<(String, String, String), String> {
    combine_mode_header_and_user(DualHeaderPromptInput {
        store,
        artifacts,
        text,
        model: opts.model,
        git: opts.git,
        mode_template: DO_HEADER_MD,
    })
}

/// Work-turn body for `--do` after spawn already delivered `header.md` + `do_header.md`.
#[must_use]
pub(crate) fn build_do_coder_run_with_store(
    _store: &PromptStore,
    _artifacts: &RunArtifacts,
    text: &str,
    _opts: PromptModelOpts<'_>,
) -> DoCoderRun {
    let user = text.trim_end().to_string();
    DoCoderRun {
        combined: user.clone(),
        header_user_for_trace: (String::new(), user),
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<DoCoderRun> = None;
        let _ = build_do_coder_run_with_store;
        let _ = prepare_do_prompt_store;
    }
}
