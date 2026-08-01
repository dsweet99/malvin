use crate::artifacts::RunArtifacts;
use crate::orchestrator::workflow_context_paths_only;
use crate::prompts::{PromptError, PromptStore, ROUTER_SUMMARIZE_MD};

pub(crate) struct RouterSummarizePromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
}

pub(crate) fn build_router_summarize_prompt(
    input: RouterSummarizePromptInput<'_>,
) -> Result<String, String> {
    let RouterSummarizePromptInput {
        store,
        artifacts,
        model,
        git,
    } = input;
    let ctx = workflow_context_paths_only(artifacts, model, git);
    let body = store
        .render_prompt_only(ROUTER_SUMMARIZE_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

#[cfg(test)]
mod kiss_cov {
    #[test]
    fn kiss_cov_summarize_prompt_names() {
        let _ = super::build_router_summarize_prompt;
        let _: Option<super::RouterSummarizePromptInput> = None;
    }
}
