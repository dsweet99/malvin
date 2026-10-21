use malvin::artifacts::RunArtifacts;
use malvin::orchestrator::workflow_context_paths_only;
use malvin::prompts::{
    PromptError, PromptStore, ROUTER_B_DONE_NOTE_MD, header_prompt_file, router_a_2_prompt_file,
    router_a_audit_prompt_file, router_a_prompt_file, router_b_prompt_file,
    router_b_satisfy_prompt_file,
};

use super::router_flow::router_flow_prompt::{RouterCodeExtraInput, render_router_code_extra};

pub(crate) struct RouterHeaderPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
}

pub(crate) fn build_router_header_prompt(
    input: RouterHeaderPromptInput<'_>,
) -> Result<String, String> {
    let ctx = workflow_context_paths_only(input.artifacts, input.model);
    let body = input
        .store
        .render_prompt_only(header_prompt_file(), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) struct RouterAPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub gates: bool,
    pub gates_just_ran: bool,
}

pub(crate) fn build_router_a_prompt(input: RouterAPromptInput<'_>) -> Result<String, String> {
    let RouterAPromptInput {
        store,
        artifacts,
        model,
        gates,
        gates_just_ran,
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
    let audit = prompt_fragment(store, router_a_audit_prompt_file())?;
    ctx.insert("audit_directive".to_string(), audit);
    let body = store
        .render_prompt_only(router_a_prompt_file(), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) fn build_router_a_2_prompt(store: &PromptStore) -> Result<String, String> {
    prompt_fragment(store, router_a_2_prompt_file())
}

pub(crate) fn build_router_c_prompt(store: &PromptStore) -> Result<String, String> {
    prompt_fragment(store, malvin::prompts::ROUTER_C_MD)
}

pub(crate) struct RouterBPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
}

fn prompt_fragment(store: &PromptStore, name: &str) -> Result<String, String> {
    store
        .render_prompt_only(name, &std::collections::HashMap::new())
        .map_err(|e: PromptError| e.0)
        .map(|body| body.trim().to_string())
}

pub(crate) fn build_router_b_prompt(input: RouterBPromptInput<'_>) -> Result<String, String> {
    let mut ctx = workflow_context_paths_only(input.artifacts, input.model);
    ctx.insert("creative_lead".to_string(), String::new());
    ctx.insert(
        "satisfy_line".to_string(),
        prompt_fragment(input.store, router_b_satisfy_prompt_file())?,
    );
    ctx.insert(
        "done_note".to_string(),
        prompt_fragment(input.store, ROUTER_B_DONE_NOTE_MD)?,
    );
    let body = input
        .store
        .render_prompt_only(router_b_prompt_file(), ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

#[must_use]
pub(crate) const fn router_b_prompt_label() -> &'static str {
    router_b_prompt_file()
}

#[must_use]
pub(crate) const fn router_a_prompt_label() -> &'static str {
    router_a_prompt_file()
}
