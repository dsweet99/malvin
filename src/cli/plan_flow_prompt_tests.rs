use std::collections::HashMap;

use crate::cli::adversarial_profile::adversarial_profile_active;
use crate::prompts::{PromptStore, PLAN_1A_RESTATE_MD, PLAN_1B_CRITIQUE_MD, PLAN_2_DECISIONS_MD, PLAN_3_REWRITE_MD};
use super::{
    build_plan_render_context, prepare_plan_prompt_store, render_plan_1a, render_plan_1b,
    render_plan_2, render_plan_3,
};

#[test]
fn plan_1a_prompt_forbids_critique_in_contract() {
    let store = prepare_plan_prompt_store().expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = build_plan_render_context(&plan, tmp.path(), &artifacts);
    let text = render_plan_1a(&store, &ctx).expect("render");
    assert!(!text.contains("{{"));
    assert!(text.contains("Do **not** fix, critique"));
    assert!(text.contains("Do **not** add `## Critique`"));
}

#[test]
fn plan_1b_prompt_critiques_user_span_not_restatement() {
    let store = prepare_plan_prompt_store().expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = build_plan_render_context(&plan, tmp.path(), &artifacts);
    let text = render_plan_1b(&store, &ctx).expect("render");
    assert!(text.contains("original user text above `---`"));
    assert!(text.contains("not the restatement"));
}

#[test]
fn plan_2_prompt_requires_decisions_append() {
    let store = prepare_plan_prompt_store().expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = build_plan_render_context(&plan, tmp.path(), &artifacts);
    let text = render_plan_2(&store, &ctx).expect("render");
    assert!(text.contains("## DECISIONS"));
    assert!(text.contains("immediately after"));
}

#[test]
fn plan_3_prompt_requires_fenced_block_not_file_edit() {
    let store = prepare_plan_prompt_store().expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = build_plan_render_context(&plan, tmp.path(), &artifacts);
    let text = render_plan_3(&store, &ctx).expect("render");
    assert!(text.contains("fenced markdown block"));
    assert!(text.contains("Do **not** edit"));
}

#[test]
fn adversarial_overlay_injected_when_profile_active() {
    let store = prepare_plan_prompt_store().expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("adversarial.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = build_plan_render_context(&plan, tmp.path(), &artifacts);
    assert!(adversarial_profile_active(&plan, tmp.path()));
    let text = render_plan_1b(&store, &ctx).expect("render");
    assert!(text.contains("smell_registry.toml"));
}

#[test]
fn embedded_plan_prompts_use_spaced_brace_placeholders() {
    let store = PromptStore::default_store();
    for name in [
        PLAN_1A_RESTATE_MD,
        PLAN_1B_CRITIQUE_MD,
        PLAN_2_DECISIONS_MD,
        PLAN_3_REWRITE_MD,
    ] {
        let text = store.prompt_text(name).expect("prompt");
        let bad = crate::prompts::malformed_brace_placeholders(&text);
        assert!(bad.is_empty(), "{name}: {bad:?}");
    }
}

#[test]
fn render_plan_prompt_expands_placeholders() {
    let store = prepare_plan_prompt_store().expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx = build_plan_render_context(&plan, tmp.path(), &artifacts);
    let text = super::render_plan_prompt(&store, crate::prompts::PLAN_2_DECISIONS_MD, &ctx)
        .expect("render");
    assert!(text.contains("## DECISIONS"));
    assert!(!text.contains("{{"));
}

#[test]
fn plan_prompt_context_expands_plan_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "body").expect("write");
    let artifacts =
        crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    let ctx: HashMap<String, String> = build_plan_render_context(&plan, tmp.path(), &artifacts);
    let store = prepare_plan_prompt_store().expect("store");
    let text = render_plan_1a(&store, &ctx).expect("render");
    assert!(!text.contains("{{ plan_path }}"));
    assert!(text.contains("plan.md"));
}
