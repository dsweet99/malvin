use crate::artifacts::RunArtifacts;
use crate::orchestrator::workflow_context_paths_only;
use crate::prompts::{
    PromptError, PromptStore, RouterBPromptFlags, header_prompt_file, kpop_common_prompt_file,
    router_a_prompt_file, router_b_prompt_file,
};

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

pub(crate) struct RouterKpopCommonPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub max_hypotheses: usize,
    pub no_kpop: bool,
}

pub(crate) fn build_router_kpop_common_prompt(
    input: RouterKpopCommonPromptInput<'_>,
) -> Result<String, String> {
    let template = kpop_common_prompt_file(input.no_kpop);
    let mut ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    ctx.insert("max_hypotheses", input.max_hypotheses.to_string());
    ctx.insert(
        "exp_log",
        crate::format_prompt_path(
            input.artifacts.gate_exp_log_path(1).as_path(),
            input.artifacts.work_dir.as_path(),
        ),
    );
    input
        .store
        .render_prompt_only(template, ctx.as_map())
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

/// Render `mbc2.md` for a creative router iteration (after `kpop_common`).
pub(crate) fn build_router_mbc2_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
) -> Result<String, String> {
    let user_prompt = std::fs::read_to_string(&artifacts.plan_path).map_err(|e| {
        format!(
            "failed to read user request for mbc2 ({}): {e}",
            artifacts.plan_path.display()
        )
    })?;
    let ctx = crate::prompts::build_mbc2_render_context(&user_prompt);
    crate::prompts::render_mbc2_prompt(store, &ctx)
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

pub(crate) struct RouterAPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub gates: bool,
    pub no_kpop: bool,
}

pub(crate) fn build_router_a_prompt(input: RouterAPromptInput<'_>) -> Result<String, String> {
    let RouterAPromptInput {
        store,
        artifacts,
        model,
        git,
        gates,
        no_kpop,
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
        .render_prompt_only(router_a_prompt_file(no_kpop), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) struct RouterBPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub creative: bool,
    pub no_kpop: bool,
}

pub(crate) fn build_router_b_prompt(input: RouterBPromptInput<'_>) -> Result<String, String> {
    let template = router_b_prompt_file(RouterBPromptFlags {
        creative: input.creative,
        no_kpop: input.no_kpop,
    });
    let ctx = workflow_context_paths_only(input.artifacts, input.model, input.git);
    let body = input
        .store
        .render_prompt_only(template, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

#[must_use]
pub(crate) const fn router_b_prompt_label(flags: RouterBPromptFlags) -> &'static str {
    router_b_prompt_file(flags)
}

#[must_use]
pub(crate) const fn kpop_common_prompt_label(no_kpop: bool) -> &'static str {
    kpop_common_prompt_file(no_kpop)
}

#[must_use]
pub(crate) const fn router_a_prompt_label(no_kpop: bool) -> &'static str {
    router_a_prompt_file(no_kpop)
}
