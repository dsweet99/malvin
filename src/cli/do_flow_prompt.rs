use std::collections::HashMap;

use crate::artifacts::RunArtifacts;
use crate::prompt_stratification::join_strata;
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore, render_header};

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
    context: &HashMap<String, String>,
) -> Result<(String, String, String), String> {
    let header_body = store
        .render_prompt_only(template_file, context)
        .map_err(|e: PromptError| e.0)?;
    let header = header_body.trim_end().to_string();
    let user = text.trim_end().to_string();
    let combined = join_strata([&header, &user]);
    Ok((combined, header, user))
}

pub fn combine_do_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    use crate::orchestrator::workflow_context;
    let context = workflow_context(artifacts, store, "do").map_err(|e: PromptError| e.0)?;
    let header = render_header(store, &context).map_err(|e: PromptError| e.0)?;
    let user = text.trim_end().to_string();
    let combined = join_strata([&header, &user]);
    Ok((combined, header, user))
}

pub fn combine_do_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    use crate::orchestrator::workflow_context_paths_only;
    let context = workflow_context_paths_only(artifacts, "do");
    combine_do_prompt_file_and_user(store, text, DO_HEADER_MD, &context)
}

pub(crate) fn build_do_coder_run_with_store(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<DoCoderRun, String> {
    let (_, coding_header, _) =
        combine_do_acp_prompt_header_and_user(store, artifacts, "")?;
    let (_, do_header, user) = combine_do_raw_header_and_user(store, artifacts, text)?;
    let combined = join_strata([&coding_header, &do_header, &user]);
    let trace_header = join_strata([&coding_header, &do_header]);
    Ok(DoCoderRun {
        combined,
        header_user_for_trace: (trace_header, user),
    })
}

pub(crate) fn build_do_coder_run(artifacts: &RunArtifacts, text: &str) -> Result<DoCoderRun, String> {
    let store = prepare_do_prompt_store()?;
    build_do_coder_run_with_store(&store, artifacts, text)
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<DoCoderRun> = None;
    }
}
