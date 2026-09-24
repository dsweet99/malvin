use malvin::artifacts::RunArtifacts;
use malvin::orchestrator::workflow_context_paths_only;
use malvin::prompts::{
    PromptError, PromptStore, ROUTER_B_CREATIVE_LEAD_MD, ROUTER_B_DONE_NOTE_MD, RouterBPromptFlags,
    header_prompt_file, kpop_common_prompt_file, router_a_audit_prompt_file, router_a_prompt_file,
    router_b_prompt_file, router_b_satisfy_prompt_file, router_b_uses_creative_lead,
    router_b_uses_done_note,
};

use super::{RouterCodeExtraInput, render_router_code_extra};

pub(crate) struct RouterHeaderPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub max_hypotheses: usize,
    pub no_kpop: bool,
    pub gate_iteration: usize,
}

pub(crate) fn build_router_header_prompt(
    input: RouterHeaderPromptInput<'_>,
) -> Result<String, String> {
    let mut ctx = workflow_context_paths_only(input.artifacts, input.model);
    let kpop = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
        store: input.store,
        artifacts: input.artifacts,
        model: input.model,
        max_hypotheses: input.max_hypotheses,
        no_kpop: input.no_kpop,
        gate_iteration: input.gate_iteration,
    })?;
    ctx.insert("kpop_insert", kpop);
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
    pub max_hypotheses: usize,
    pub no_kpop: bool,
    pub gate_iteration: usize,
}

pub(crate) fn build_router_kpop_common_prompt(
    input: RouterKpopCommonPromptInput<'_>,
) -> Result<String, String> {
    let template = kpop_common_prompt_file(input.no_kpop);
    let mut ctx = workflow_context_paths_only(input.artifacts, input.model);
    ctx.insert("max_hypotheses", input.max_hypotheses.to_string());
    let iteration = if input.gate_iteration == 0 {
        1
    } else {
        input.gate_iteration
    };
    ctx.insert(
        "exp_log",
        malvin::format_prompt_path(
            input.artifacts.gate_exp_log_path(iteration).as_path(),
            input.artifacts.work_dir.as_path(),
        ),
    );
    input
        .store
        .render_prompt_only(template, ctx.as_map())
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

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
    let ctx = malvin::prompts::build_mbc2_render_context(&user_prompt);
    malvin::prompts::render_mbc2_prompt(store, &ctx)
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

pub(crate) struct RouterAPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub gates: bool,
    pub gates_just_ran: bool,
    pub no_kpop: bool,
}

pub(crate) fn build_router_a_prompt(input: RouterAPromptInput<'_>) -> Result<String, String> {
    let RouterAPromptInput {
        store,
        artifacts,
        model,
        gates,
        gates_just_ran,
        no_kpop,
    } = input;
    let mut ctx = workflow_context_paths_only(artifacts, model);
    let code_extra = render_router_code_extra(RouterCodeExtraInput {
        store,
        artifacts,
        model,
        gates,
        gates_just_ran,
    })?;
    ctx.insert("code_extra".to_string(), code_extra);
    let audit = prompt_fragment(store, router_a_audit_prompt_file(no_kpop))?;
    ctx.insert("audit_directive".to_string(), audit);
    let body = store
        .render_prompt_only(router_a_prompt_file(no_kpop), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) struct RouterBPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub creative: bool,
    pub no_kpop: bool,
}

fn prompt_fragment(store: &PromptStore, name: &str) -> Result<String, String> {
    store
        .render_prompt_only(name, &std::collections::HashMap::new())
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

fn router_b_creative_lead(store: &PromptStore, flags: RouterBPromptFlags) -> Result<String, String> {
    if !router_b_uses_creative_lead(flags) {
        return Ok(String::new());
    }
    let lead = prompt_fragment(store, ROUTER_B_CREATIVE_LEAD_MD)?;
    if lead.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("{lead}\n\n"))
    }
}

fn router_b_done_note(store: &PromptStore, flags: RouterBPromptFlags) -> Result<String, String> {
    if router_b_uses_done_note(flags) {
        prompt_fragment(store, ROUTER_B_DONE_NOTE_MD)
    } else {
        Ok(String::new())
    }
}

fn router_b_template_keys(
    store: &PromptStore,
    flags: RouterBPromptFlags,
) -> Result<(String, String, String), String> {
    Ok((
        router_b_creative_lead(store, flags)?,
        prompt_fragment(store, router_b_satisfy_prompt_file(flags))?,
        router_b_done_note(store, flags)?,
    ))
}

pub(crate) fn build_router_b_prompt(input: RouterBPromptInput<'_>) -> Result<String, String> {
    let flags = RouterBPromptFlags {
        creative: input.creative,
        no_kpop: input.no_kpop,
    };
    let mut ctx = workflow_context_paths_only(input.artifacts, input.model);
    let (creative_lead, satisfy_line, done_note) = router_b_template_keys(input.store, flags)?;
    ctx.insert("creative_lead".to_string(), creative_lead);
    ctx.insert("satisfy_line".to_string(), satisfy_line);
    ctx.insert("done_note".to_string(), done_note);
    let body = input
        .store
        .render_prompt_only(router_b_prompt_file(flags), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

#[must_use]
pub(crate) const fn router_b_prompt_label(flags: RouterBPromptFlags) -> &'static str {
    router_b_prompt_file(flags)
}

#[must_use]
pub(crate) const fn router_a_prompt_label(no_kpop: bool) -> &'static str {
    router_a_prompt_file(no_kpop)
}
