use crate::artifacts::RunArtifacts;
use crate::orchestrator::workflow_context_paths_only;
use crate::prompts::{
    KPOP_COMMON_MD, PromptError, PromptStore, header_prompt_file, router_a_prompt_file,
    router_b_prompt_file,
};

use super::RouterKpopCommonPromptInput;
use super::{RouterCodeExtraInput, render_router_code_extra};

pub(crate) struct RouterHeaderPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
}

pub(crate) fn build_router_header_prompt(
    input: RouterHeaderPromptInput<'_>,
) -> Result<String, String> {
    let ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let body = input
        .store
        .render_prompt_only(header_prompt_file(), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) fn build_router_kpop_common_prompt(
    input: RouterKpopCommonPromptInput<'_>,
) -> Result<String, String> {
    let (store, artifacts, model, git, max_hypotheses) = input;
    let mut ctx = workflow_context_paths_only(artifacts, model, git);
    ctx.insert("max_hypotheses", max_hypotheses.to_string());
    ctx.insert(
        "exp_log",
        crate::format_prompt_path(
            artifacts.gate_exp_log_path(1).as_path(),
            artifacts.work_dir.as_path(),
        ),
    );
    store
        .render_prompt_only(KPOP_COMMON_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

pub(crate) struct RouterAPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub gates: bool,
}

pub(crate) fn build_router_a_prompt(input: RouterAPromptInput<'_>) -> Result<String, String> {
    let RouterAPromptInput {
        store,
        artifacts,
        model,
        git,
        gates,
    } = input;
    let mut ctx = workflow_context_paths_only(artifacts, model, git);
    let code_extra = render_router_code_extra(RouterCodeExtraInput {
        store,
        artifacts,
        model,
        git,
        gates,
    })?;
    ctx.insert("code_extra".to_string(), code_extra);
    let body = store
        .render_prompt_only(router_a_prompt_file(), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) struct RouterBPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub creative: bool,
}

pub(crate) fn build_router_b_prompt(input: RouterBPromptInput<'_>) -> Result<String, String> {
    let template = router_b_prompt_file(input.creative);
    let ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let body = input
        .store
        .render_prompt_only(template, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

#[must_use]
pub(crate) const fn router_b_prompt_label(creative: bool) -> &'static str {
    router_b_prompt_file(creative)
}
