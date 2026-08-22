use crate::config::DEFAULT_CLI_MODEL;
use crate::flow_prompt_join_test_helpers::flow_test_artifacts;
use crate::prompts::PromptStore;
use crate::router_flow::router_flow_prompt::{
    RouterAPromptInput, RouterBPromptInput, RouterSummarizePromptInput, build_router_a_prompt,
    build_router_b_prompt, build_router_header_prompt, build_router_summarize_prompt,
    prepare_router_prompt_store, router_b_prompt_label,
};

#[test]
fn build_router_a_prompt_expands_malvin_command_with_active_model() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prompt_root = tmp.path().join("prompts");
    std::fs::create_dir_all(&prompt_root).expect("mkdir prompts");
    std::fs::write(
        prompt_root.join("router_a.md"),
        "Use {{ malvin_command }}\n{{ code_extra }}\nSee {{ user_request_path }}.\n",
    )
    .expect("write router_a");
    std::fs::write(
        prompt_root.join("router_code_extra.md"),
        "- Make sure the code checks all pass:\n```\n{{ code_checks }}\n```\n",
    )
    .expect("write code_extra");
    let store = PromptStore::with_root(prompt_root);
    let artifacts = flow_test_artifacts(&tmp);
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: "composer-2",
        git: false,
        gates: false,
    })
    .expect("router_a");
    assert!(body.contains("malvin --model=composer-2"));
    assert!(!body.contains("{{ malvin_command }}"));
}

#[test]
fn build_router_a_prompt_renders_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_a");
    assert!(!body.contains("{{"));
    assert!(body.contains("__MALVIN_DONE__"));
}

#[test]
fn build_router_a_prompt_includes_code_checks_when_gates_enabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: true,
    })
    .expect("router_a");
    assert!(body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("{{"));
}

#[test]
fn build_router_a_prompt_omits_code_checks_when_gates_disabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "echo ROUTER_CHECK_LINE\n");
    let store = prepare_router_prompt_store().expect("store");
    crate::gate_loop_session::set_quality_gates_just_ran(true);
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_a");
    crate::gate_loop_session::set_quality_gates_just_ran(false);
    assert!(!body.contains("echo ROUTER_CHECK_LINE"));
    assert!(!body.contains("quality gates were just run"));
    assert!(!body.contains("{{"));
}

#[test]
fn router_code_extra_note_absent_when_gates_have_not_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let store = prepare_router_prompt_store().expect("store");
    crate::gate_loop_session::set_quality_gates_just_ran(false);
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: true,
    })
    .expect("router_a");
    assert!(
        !body.contains("quality gates were just run"),
        "note must be absent before any gate run: {body}"
    );
    assert!(!body.contains("{{"));
}

#[test]
fn router_code_extra_note_present_after_gates_just_ran() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let store = prepare_router_prompt_store().expect("store");
    crate::gate_loop_session::set_quality_gates_just_ran(true);
    let body = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: true,
    })
    .expect("router_a");
    crate::gate_loop_session::set_quality_gates_just_ran(false);
    assert!(
        body.contains("The quality gates were just run, and their output is in `"),
        "note must be present right after a gate run: {body}"
    );
    assert!(
        body.contains("quality_gates.log"),
        "note must name the quality_gates.log path: {body}"
    );
    assert!(!body.contains("{{"));
    assert!(
        !body.contains("NB: The code checks may have already been run"),
        "old unconditional NB line must be gone: {body}"
    );
}

#[test]
fn build_router_summarize_prompt_renders_dm_body_without_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let body = build_router_summarize_prompt(RouterSummarizePromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
    })
    .expect("router_summarize");
    assert!(!body.contains("{{"));
    assert!(
        body.contains("Write a summary of this entire session"),
        "must render router_summarize.md: {body}"
    );
}

#[test]
fn build_router_b_prompt_selects_creative_template_when_flag_set() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let plain = build_router_b_prompt(RouterBPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        creative: false,
    })
    .expect("router_b");
    let creative = build_router_b_prompt(RouterBPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        creative: true,
    })
    .expect("router_b_creative");
    assert!(
        plain.contains("KPop: Satisfy the requirements."),
        "default router_b must keep KPop satisfy instruction: {plain}"
    );
    assert!(
        !plain.contains("malvin inspire"),
        "default router_b must not mention inspire: {plain}"
    );
    assert!(creative.contains("malvin inspire"));
    assert!(
        creative.contains("KPop: Satisfy the requirements."),
        "creative router_b must keep KPop satisfy instruction: {creative}"
    );
    assert_eq!(router_b_prompt_label(false), "router_b.md");
    assert_eq!(router_b_prompt_label(true), "router_b_creative.md");
}

#[test]
fn build_router_prompts_use_canonical_templates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = flow_test_artifacts(&tmp);
    let store = prepare_router_prompt_store().expect("store");
    let header = build_router_header_prompt(
        crate::router_flow::router_flow_prompt::RouterHeaderPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
        },
    )
    .expect("header");
    assert!(!header.to_ascii_lowercase().contains("falsifiable"));
    let a = build_router_a_prompt(RouterAPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        gates: false,
    })
    .expect("router_a");
    assert!(!a.to_ascii_lowercase().contains("falsif"));
    let b = build_router_b_prompt(RouterBPromptInput {
        store: &store,
        artifacts: &artifacts,
        model: DEFAULT_CLI_MODEL,
        git: false,
        creative: false,
    })
    .expect("router_b");
    assert!(!b.to_ascii_lowercase().contains("falsif"));
}
