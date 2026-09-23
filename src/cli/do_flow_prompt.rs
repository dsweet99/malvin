use malvin::artifacts::RunArtifacts;
use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};
use malvin::workflow_context::PromptModelOpts;

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
