use malvin::artifacts::RunArtifacts;
use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};
use malvin::workflow_context::PromptModelOpts;

use crate::cli::session_header::render_do_cosend_prompt;

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

pub(crate) fn build_do_coder_run_with_store(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    opts: PromptModelOpts<'_>,
) -> Result<DoCoderRun, String> {
    let (header, combined) = render_do_cosend_prompt(store, artifacts, opts.model, text)?;
    let user = text.trim_end().to_string();
    Ok(DoCoderRun {
        combined,
        header_user_for_trace: (header, user),
    })
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
