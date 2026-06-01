//! Prompt rendering for `malvin plan`.

use std::collections::HashMap;
use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::cli::adversarial_profile::adversarial_overlay_hint;
use crate::prompts::{
    PLAN_1A_RESTATE_MD, PLAN_1B_CRITIQUE_MD, PLAN_2_DECISIONS_MD, PLAN_3_REWRITE_MD, PromptError,
    PromptStore,
};
use crate::workflow_context::insert_formatted;

const ADVERSARIAL_OVERLAY_BODY: &str = r"**Adversarial profile (active):** Map plan bullets to `smell_registry.toml` row ids where applicable. Flag missing MR/PBT obligations, null-plugin non-vacuity, and materialization-harness gaps. Prompt 3 must link smell-registry row ids (`principle`, `invariant`, `counterexample_shape`), MR/PBT obligation class per row, and materialization-harness milestones.";

pub fn prepare_plan_prompt_store() -> Result<PromptStore, String> {
    let store = PromptStore::default_store();
    store.ensure_defaults().map_err(|e: PromptError| e.0)?;
    for name in [
        PLAN_1A_RESTATE_MD,
        PLAN_1B_CRITIQUE_MD,
        PLAN_2_DECISIONS_MD,
        PLAN_3_REWRITE_MD,
    ] {
        store.validate_exists(name).map_err(|e: PromptError| e.0)?;
    }
    Ok(store)
}

pub(crate) fn build_plan_render_context(
    source_plan_path: &Path,
    work_dir: &Path,
    artifacts: &RunArtifacts,
) -> HashMap<String, String> {
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "plan_path", source_plan_path, work_dir);
    let overlay = adversarial_overlay_hint(source_plan_path, work_dir)
        .map(|reason| format!("{ADVERSARIAL_OVERLAY_BODY}\n\nActivation: {reason}"))
        .unwrap_or_default();
    ctx.insert("adversarial_overlay".to_string(), overlay);
    ctx.insert(
        "malvin_command".to_string(),
        "plan".to_string(),
    );
    let _ = artifacts;
    ctx
}

pub(crate) fn render_plan_prompt(
    store: &PromptStore,
    template: &str,
    ctx: &HashMap<String, String>,
) -> Result<String, String> {
    store
        .render_prompt_only(template, ctx)
        .map_err(|e: PromptError| e.0)
}

pub(crate) fn render_plan_1a(store: &PromptStore, ctx: &HashMap<String, String>) -> Result<String, String> {
    render_plan_prompt(store, PLAN_1A_RESTATE_MD, ctx)
}

pub(crate) fn render_plan_1b(store: &PromptStore, ctx: &HashMap<String, String>) -> Result<String, String> {
    render_plan_prompt(store, PLAN_1B_CRITIQUE_MD, ctx)
}

pub(crate) fn render_plan_2(store: &PromptStore, ctx: &HashMap<String, String>) -> Result<String, String> {
    render_plan_prompt(store, PLAN_2_DECISIONS_MD, ctx)
}

pub(crate) fn render_plan_3(store: &PromptStore, ctx: &HashMap<String, String>) -> Result<String, String> {
    render_plan_prompt(store, PLAN_3_REWRITE_MD, ctx)
}

#[cfg(test)]
#[path = "plan_flow_prompt_tests.rs"]
mod plan_flow_prompt_tests;
