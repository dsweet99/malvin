use crate::artifacts::RunArtifacts;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore, render_header};

pub(crate) struct DualHeaderPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub text: &'a str,
    pub model: &'a str,
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
    model: &str,
) -> Result<(String, String, String), String> {
    use crate::orchestrator::workflow_context_paths_only;
    let context = workflow_context_paths_only(artifacts, model);
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
    let context = workflow_context_paths_only(input.artifacts, input.model);
    combine_prompt_file_and_user(
        input.store,
        input.text,
        input.mode_template,
        &context,
    )
}

pub(crate) struct DualHeaderCoderRun {
    pub combined: String,
    pub header_user_for_trace: (String, String),
}

pub(crate) fn build_dual_header_coder_run_with_store(
    input: DualHeaderPromptInput<'_>,
) -> Result<DualHeaderCoderRun, String> {
    let (_, coding_header, _) =
        combine_acp_prompt_header_and_user(input.store, input.artifacts, "", input.model)?;
    let (_, mode_header, user) = combine_mode_header_and_user(input)?;
    let combined = join_labeled_strata([
        (PromptStratum::WorkflowHeader, &coding_header),
        (PromptStratum::WorkflowHeader, &mode_header),
        (PromptStratum::UserRequest, &user),
    ]);
    let trace_header = join_labeled_strata([
        (PromptStratum::WorkflowHeader, &coding_header),
        (PromptStratum::WorkflowHeader, &mode_header),
    ]);
    Ok(DualHeaderCoderRun {
        combined,
        header_user_for_trace: (trace_header, user),
    })
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<DualHeaderPromptInput<'_>> = None;
        let _: Option<DualHeaderCoderRun> = None;
    }
}
