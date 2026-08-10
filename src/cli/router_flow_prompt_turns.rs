use crate::artifacts::RunArtifacts;
use crate::cli::flow_prompt_combine::combine_acp_prompt_header_and_user;
use crate::orchestrator::workflow_context_paths_only;
use crate::prompts::{PromptError, PromptStore, ROUTER_A_MD, ROUTER_B_MD};
use crate::workflow_context::PromptModelOpts;
use std::path::Path;

use super::{
    format_exp_log_for_prompt, render_router_code_extra, RouterCodeExtraInput,
};

pub(crate) struct RouterHeaderPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
}

pub(crate) fn build_router_header_prompt(
    input: RouterHeaderPromptInput<'_>,
) -> Result<String, String> {
    let (_, header, _) = combine_acp_prompt_header_and_user(
        input.store,
        input.artifacts,
        "",
        PromptModelOpts::new(input.model, input.git),
    )?;
    Ok(header)
}

pub(crate) struct RouterKpopCommonPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub max_hypotheses: usize,
    pub exp_log: &'a Path,
}

pub(crate) fn build_router_kpop_common_prompt(
    input: RouterKpopCommonPromptInput<'_>,
) -> Result<String, String> {
    let mut ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let base = input.artifacts.work_dir.as_path();
    ctx.insert(
        "exp_log".to_string(),
        format_exp_log_for_prompt(input.exp_log, base),
    );
    ctx.insert(
        "max_hypotheses".to_string(),
        input.max_hypotheses.to_string(),
    );
    input
        .store
        .render_prompt_only("kpop_common.md", ctx.as_map())
        .map_err(|e: PromptError| e.0)
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
        .render_prompt_only(ROUTER_A_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) struct RouterBPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
}

pub(crate) fn build_router_b_prompt(input: RouterBPromptInput<'_>) -> Result<String, String> {
    let ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let body = input
        .store
        .render_prompt_only(ROUTER_B_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}
