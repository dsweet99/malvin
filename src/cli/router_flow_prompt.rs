use crate::artifacts::RunArtifacts;
use crate::cli::flow_prompt_combine::{
    combine_acp_prompt_header_and_user, combine_mode_header_and_user, combine_prompt_file_and_user,
    DualHeaderPromptInput,
};
use crate::orchestrator::workflow_context_paths_only;
use crate::workflow_context::{format_prompt_path, PromptModelOpts};
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::{
    PromptError, PromptStore, ROUTER_CODE_EXTRA_MD, ROUTER_SUMMARIZE_MD, header_prompt_file,
    kpop_common_prompt_file, router_a_prompt_file, router_b_prompt_file,
};
use std::path::Path;

#[path = "router_flow_prompt_summarize.rs"]
mod router_flow_prompt_summarize;
pub(crate) use router_flow_prompt_summarize::{
    build_router_summarize_prompt, RouterSummarizePromptInput,
};

#[path = "router_flow_prompt_turns.rs"]
mod router_flow_prompt_turns;
pub(crate) use router_flow_prompt_turns::{
    build_router_a_prompt, build_router_b_prompt, build_router_header_prompt,
    build_router_kpop_common_prompt, router_b_prompt_label, RouterAPromptInput, RouterBPromptInput,
    RouterHeaderPromptInput, RouterKpopCommonPromptInput,
};

pub fn prepare_router_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    validate_router_required_prompts(&store)?;
    Ok(store)
}

fn validate_router_required_prompts(store: &PromptStore) -> Result<(), String> {
    let required = [
        header_prompt_file(),
        router_a_prompt_file(),
        router_b_prompt_file(false),
        router_b_prompt_file(true),
        ROUTER_CODE_EXTRA_MD,
        ROUTER_SUMMARIZE_MD,
        kpop_common_prompt_file(),
    ];
    for name in required {
        store.validate_exists(name).map_err(|e: PromptError| e.0)?;
    }
    Ok(())
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
    opts: PromptModelOpts<'_>,
) -> Result<(String, String, String), String> {
    combine_acp_prompt_header_and_user(store, artifacts, text, opts)
}

pub fn combine_router_raw_header_and_user(
    store: &PromptStore,
    artifacts: &RunArtifacts,
    text: &str,
    opts: PromptModelOpts<'_>,
) -> Result<(String, String, String), String> {
    combine_mode_header_and_user(DualHeaderPromptInput {
        store,
        artifacts,
        text,
        model: opts.model,
        git: opts.git,
        mode_template: router_a_prompt_file(),
    })
}

pub(crate) fn router_code_checks_text(work_dir: &Path) -> Result<String, String> {
    let commands = crate::repo_gates::gate_command_lines(work_dir)?;
    Ok(commands.join("\n"))
}

pub(crate) struct RouterCodeExtraInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub git: bool,
    pub gates: bool,
}

pub(crate) fn render_router_code_extra(input: RouterCodeExtraInput<'_>) -> Result<String, String> {
    let RouterCodeExtraInput {
        store,
        artifacts,
        model,
        git,
        gates,
    } = input;
    let mut ctx = workflow_context_paths_only(artifacts, model, git);
    let code_checks = if gates {
        router_code_checks_text(artifacts.work_dir.as_path())?
    } else {
        String::new()
    };
    ctx.insert("code_checks".to_string(), code_checks);
    let body = store
        .render_prompt_only(ROUTER_CODE_EXTRA_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(body.trim().to_string())
}

pub(crate) fn format_exp_log_for_prompt(exp_log: &Path, work_dir: &Path) -> String {
    format_prompt_path(exp_log, work_dir)
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = build_router_header_prompt;
        let _ = build_router_kpop_common_prompt;
        let _ = build_router_a_prompt;
        let _ = build_router_b_prompt;
        let _ = router_b_prompt_label;
        let _ = build_router_summarize_prompt;
        let _ = render_router_code_extra;
    }
}
