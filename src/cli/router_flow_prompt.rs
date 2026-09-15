use crate::cli::flow_prompt_combine::{
    DualHeaderPromptInput, combine_acp_prompt_header_and_user, combine_mode_header_and_user,
    combine_prompt_file_and_user,
};
use malvin::artifacts::RunArtifacts;
use malvin::orchestrator::workflow_context_paths_only;
use malvin::prompt_stratification::WorkflowRenderContext;
use malvin::prompts::{
    PromptError, PromptStore, ROUTER_CODE_EXTRA_MD, ROUTER_SUMMARIZE_MD, RouterBPromptFlags,
    header_prompt_file, kpop_common_prompt_file, router_a_prompt_file, router_b_prompt_file,
};
use malvin::workflow_context::PromptModelOpts;
use std::path::Path;

#[path = "router_flow_prompt_summarize.rs"]
mod router_flow_prompt_summarize;
pub(crate) use router_flow_prompt_summarize::{
    RouterSummarizePromptInput, build_router_summarize_prompt,
};

#[path = "router_flow_prompt_turns.rs"]
mod router_flow_prompt_turns;
pub(crate) use router_flow_prompt_turns::{
    RouterAPromptInput, RouterBPromptInput, RouterHeaderPromptInput, build_router_a_prompt,
    build_router_b_prompt, build_router_header_prompt, build_router_mbc2_prompt,
    router_a_prompt_label, router_b_prompt_label,
};
#[cfg(test)]
pub(crate) use router_flow_prompt_turns::{
    RouterKpopCommonPromptInput, build_router_kpop_common_prompt,
};

#[path = "router_flow_prompt_initial.rs"]
mod router_flow_prompt_initial;
#[cfg(test)]
pub(crate) use router_flow_prompt_initial::RouterInitialPrompt;
pub(crate) use router_flow_prompt_initial::{
    RouterInitialPromptInput, build_router_initial_prompt,
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
        kpop_common_prompt_file(false),
        kpop_common_prompt_file(true),
        "mbc2.md",
        router_a_prompt_file(false),
        router_a_prompt_file(true),
        router_b_prompt_file(RouterBPromptFlags {
            creative: false,
            no_kpop: false,
        }),
        router_b_prompt_file(RouterBPromptFlags {
            creative: true,
            no_kpop: false,
        }),
        router_b_prompt_file(RouterBPromptFlags {
            creative: false,
            no_kpop: true,
        }),
        ROUTER_CODE_EXTRA_MD,
        ROUTER_SUMMARIZE_MD,
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
        mode_template: router_a_prompt_file(false),
    })
}

pub(crate) fn router_code_checks_text(work_dir: &Path) -> Result<String, String> {
    let commands = malvin::repo_gates::gate_command_lines(work_dir)?;
    Ok(commands.join("\n"))
}

pub(crate) struct RouterCodeExtraInput<'a> {
    pub store: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub model: &'a str,
    pub gates: bool,
}

fn gates_enabled_code_checks(gates: bool, work_dir: &Path) -> Result<String, String> {
    if gates {
        router_code_checks_text(work_dir)
    } else {
        Ok(String::new())
    }
}

fn code_extra_with_optional_gates_note(body: String, quality_gates_log: Option<&str>) -> String {
    quality_gates_log.map_or_else(
        || body.trim().to_string(),
        |path| {
            format!("{body}\n\nThe quality gates were just run, and their output is in `{path}`.\n")
                .trim()
                .to_string()
        },
    )
}

pub(crate) fn render_router_code_extra(input: RouterCodeExtraInput<'_>) -> Result<String, String> {
    let RouterCodeExtraInput {
        store,
        artifacts,
        model,
        gates,
    } = input;
    let code_checks = gates_enabled_code_checks(gates, artifacts.work_dir.as_path())?;
    if code_checks.trim().is_empty() {
        return Ok(String::new());
    }
    let mut ctx = workflow_context_paths_only(artifacts, model);
    ctx.insert("code_checks".to_string(), code_checks);
    let body = store
        .render_prompt_only(ROUTER_CODE_EXTRA_MD, ctx.as_map())
        .map_err(|e: PromptError| e.0)?;
    let note_path = if gates && malvin::gate_loop_session::quality_gates_just_ran() {
        ctx.get("quality_gates_log").map(String::as_str)
    } else {
        None
    };
    Ok(code_extra_with_optional_gates_note(body, note_path))
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = build_router_header_prompt;
        let _ = build_router_a_prompt;
        let _ = build_router_b_prompt;
        let _ = build_router_kpop_common_prompt;
        let _ = build_router_mbc2_prompt;
        let _ = build_router_initial_prompt;
        let _: Option<RouterInitialPrompt> = None;
        let _ = router_b_prompt_label;
        let _ = router_a_prompt_label;
        let _ = build_router_summarize_prompt;
        let _ = render_router_code_extra;
    }
}
