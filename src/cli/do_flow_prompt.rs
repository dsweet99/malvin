use std::collections::HashMap;

use crate::artifacts::RunArtifacts;
use crate::prompts::{DO_HEADER_MD, HEADER_MD, PromptError, PromptStore};

pub(crate) struct DoCoderRun {
    pub combined: String,
    pub header_user_for_trace: (String, String),
    pub skip_repo_style: bool,
}

fn prepare_do_prompt_store_validating(required_template: &str) -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(required_template)
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn prepare_do_prompt_store() -> Result<PromptStore, String> {
    prepare_do_prompt_store_validating(HEADER_MD)
}

pub fn prepare_do_raw_prompt_store() -> Result<PromptStore, String> {
    prepare_do_prompt_store_validating(DO_HEADER_MD)
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
    let combined = format!("{header}\n\n{user}");
    Ok((combined, header, user))
}

pub fn combine_do_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<(String, String, String), String> {
    use crate::orchestrator::workflow_context;
    let context = workflow_context(artifacts, store, "do").map_err(|e: PromptError| e.0)?;
    combine_do_prompt_file_and_user(store, text, HEADER_MD, &context)
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

pub fn build_do_coder_run(
    cooked: bool,
    artifacts: &RunArtifacts,
    text: &str,
) -> Result<DoCoderRun, String> {
    let skip_repo_style = !cooked;
    let (combined, header_user) = if cooked {
        let store = prepare_do_prompt_store()?;
        let (combined, header, user) = combine_do_acp_prompt_header_and_user(&store, artifacts, text)?;
        (combined, (header, user))
    } else {
        let store = prepare_do_raw_prompt_store()?;
        let (combined, header, user) = combine_do_raw_header_and_user(&store, artifacts, text)?;
        (combined, (header, user))
    };
    Ok(DoCoderRun {
        combined,
        header_user_for_trace: header_user,
        skip_repo_style,
    })
}
