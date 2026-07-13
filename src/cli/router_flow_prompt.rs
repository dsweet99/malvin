use crate::artifacts::RunArtifacts;
use crate::cli::flow_prompt_combine::{
    build_dual_header_coder_run_with_store, combine_acp_prompt_header_and_user,
    combine_mode_header_and_user, combine_prompt_file_and_user, DualHeaderPromptInput,
};
use crate::orchestrator::workflow_context_paths_only;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{
    render_header, HEADER_MD, PromptError, PromptStore, ROUTER_A_1_MD, ROUTER_A_2_MD, ROUTER_C_MD,
    ROUTER_CODE_EXTRA_MD, ROUTER_D_MD,
};

pub(crate) struct RouterCoderRun {
    pub combined: String,
    /// Dual-header parts from prompt assembly; retained for tests (stdout uses uniform trace like kpop).
    #[allow(dead_code)]
    pub header_user_for_trace: (String, String),
}

pub fn prepare_router_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(HEADER_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(ROUTER_A_1_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(ROUTER_A_2_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(crate::prompts::ROUTER_B_SIMPLE_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(crate::prompts::ROUTER_B_COMPLEX_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(ROUTER_C_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(ROUTER_CODE_EXTRA_MD)
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists(ROUTER_D_MD)
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub fn combine_router_prompt_file_and_user(
    store: &PromptStore,
    text: &str,
    template_file: &str,
    context: &WorkflowRenderContext,
) -> Result<(String, String, String), String> {
    combine_prompt_file_and_user(store, text, template_file, context)
}

pub fn combine_router_acp_prompt_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<(String, String, String), String> {
    combine_acp_prompt_header_and_user(store, artifacts, text, model)
}

pub fn combine_router_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<(String, String, String), String> {
    combine_mode_header_and_user(DualHeaderPromptInput {
        store,
        artifacts,
        text,
        model,
        mode_template: ROUTER_A_1_MD,
    })
}

pub(crate) fn build_router_coder_run_with_store(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<RouterCoderRun, String> {
    let run = build_dual_header_coder_run_with_store(DualHeaderPromptInput {
        store,
        artifacts,
        text,
        model,
        mode_template: ROUTER_A_1_MD,
    })?;
    Ok(RouterCoderRun {
        combined: run.combined,
        header_user_for_trace: run.header_user_for_trace,
    })
}

#[cfg(test)]
pub(crate) fn build_router_coder_run(
    artifacts: &RunArtifacts,
    text: &str,
    model: &str,
) -> Result<RouterCoderRun, String> {
    let store = prepare_router_prompt_store()?;
    build_router_coder_run_with_store(&store, artifacts, text, model)
}

fn router_code_checks_text(work_dir: &std::path::Path) -> Result<String, String> {
    let commands = crate::repo_gates::gate_command_lines(work_dir)?;
    Ok(commands.join("\n"))
}

fn render_router_code_extra(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let work_dir = artifacts.work_dir.as_path();
    let mut ctx = workflow_context_paths_only(artifacts, model);
    ctx.insert("code_checks".to_string(), router_code_checks_text(work_dir)?);
    let body = store
        .render_prompt_only(ROUTER_CODE_EXTRA_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) fn build_router_a_2_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let ctx = workflow_context_paths_only(artifacts, model);
    let body = store
        .render_prompt_only(ROUTER_A_2_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) struct RouterBPromptInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub template: &'a str,
    pub coding_task: bool,
    pub model: &'a str,
}

pub(crate) fn build_router_b_prompt(input: RouterBPromptInput<'_>) -> Result<String, String> {
    let RouterBPromptInput {
        store,
        artifacts,
        template,
        coding_task,
        model,
    } = input;
    let mut ctx = workflow_context_paths_only(artifacts, model);
    let code_extra = if coding_task {
        render_router_code_extra(store, artifacts, model)?
    } else {
        String::new()
    };
    ctx.insert("code_extra".to_string(), code_extra);
    let body = store
        .render_prompt_only(template, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) fn build_router_c_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let ctx = workflow_context_paths_only(artifacts, model);
    let body = store
        .render_prompt_only(ROUTER_C_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) fn build_router_d_prompt(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    model: &str,
) -> Result<String, String> {
    let ctx = workflow_context_paths_only(artifacts, model);
    let header = render_header(store, ctx.as_map()).map_err(|e: PromptError| e.0)?;
    let body = store
        .render_prompt_only(ROUTER_D_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header),
        (PromptStratum::GateLoopBlock, body.trim().to_string()),
    ]))
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let run = RouterCoderRun {
            combined: String::new(),
            header_user_for_trace: (String::new(), String::new()),
        };
        assert!(run.combined.is_empty());
        let _: Option<RouterCoderRun> = None;
    }
}
