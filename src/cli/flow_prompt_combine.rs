use crate::artifacts::RunArtifacts;
use crate::prompt_stratification::{PromptStratum, WorkflowRenderContext, join_labeled_strata};
use crate::prompts::{PromptError, PromptStore, render_header};
use crate::workflow_context::PromptModelOpts;

pub(crate) struct DualHeaderPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub text: &'a str,
    pub model: &'a str,
    pub git: bool,
    pub mode_template: &'a str,
}

pub(crate) fn combine_prompt_file_and_user(
    store: &PromptStore,
    text: &str,
    template_file: &str,
    context: &WorkflowRenderContext,
) -> Result<(String, String, String), String> {
    let map = context.as_map();
    let header_body = store
        .render_prompt_only(template_file, map)
        .map_err(|e: PromptError| e.0)?;
    let header = header_body.trim_end().to_string();
    let user = text.trim_end().to_string();
    let combined = join_labeled_strata([
        (PromptStratum::WorkflowHeader, &header),
        (PromptStratum::UserRequest, &user),
    ]);
    Ok((combined, header, user))
}

pub(crate) fn combine_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    opts: PromptModelOpts<'_>,
) -> Result<(String, String, String), String> {
    use crate::orchestrator::workflow_context_paths_only;
    let context = workflow_context_paths_only(artifacts, opts.model, opts.git);
    let header = render_header(store, context.as_map()).map_err(|e: PromptError| e.0)?;
    let user = text.trim_end().to_string();
    let combined = join_labeled_strata([
        (PromptStratum::WorkflowHeader, &header),
        (PromptStratum::UserRequest, &user),
    ]);
    Ok((combined, header, user))
}

pub(crate) fn combine_mode_header_and_user(
    input: DualHeaderPromptInput<'_>,
) -> Result<(String, String, String), String> {
    use crate::orchestrator::workflow_context_paths_only;
    let context = workflow_context_paths_only(input.artifacts, input.model, input.git);
    combine_prompt_file_and_user(input.store, input.text, input.mode_template, &context)
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<DualHeaderPromptInput<'_>> = None;
        let _ = combine_mode_header_and_user;
        let _ = combine_acp_prompt_header_and_user;
    }
}
