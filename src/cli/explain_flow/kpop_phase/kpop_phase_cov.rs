use super::*;
use crate::artifacts::{explain_phase_exp_log_path, SessionDotfileBackups};

#[test]
fn chat_rules_and_phase_names() {
    assert_eq!(explain_kpop_chat_rules(EXPLAIN_PHASE_REVIEW), REVIEW_CHAT_RULES);
    assert_eq!(explain_kpop_chat_rules(EXPLAIN_PHASE_PLAN), PLAN_CHAT_RULES);
    assert!(REVIEW_CHAT_RULES.contains("LGTM"));
    assert!(PLAN_CHAT_RULES.contains("plan only"));
    assert_ne!(EXPLAIN_PHASE_REVIEW, EXPLAIN_PHASE_PLAN);
    let _ = run_explain_kpop_phase;
    let _ = run_explain_kpop_phase_once;
}

#[test]
fn prompt_builds() {
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("a");
    let context = crate::cli::workflow_kpop_shared::kpop_workflow_context_without_gates(
        &artifacts,
        crate::config::DEFAULT_CLI_MODEL,
        false,
    )
    .expect("ctx");
    let prepared = crate::kpop_engine::KPopEnginePrepared {
        artifacts,
        context,
        request_text: String::from("r"),
        startup_emit_request: String::from("r"),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    let exp = explain_phase_exp_log_path(prepared.artifacts(), 1, EXPLAIN_PHASE_REVIEW);
    let prompt = build_explain_kpop_phase_prompt(ExplainKpopPromptInput {
        prepared: &prepared,
        request_text: "body",
        exp_log_path: &exp,
        outer_iteration: 1,
        phase: EXPLAIN_PHASE_REVIEW,
        max_hypotheses: 2,
    })
    .expect("prompt");
    assert!(!prompt.is_empty());
}

#[test]
fn phase_structs() {
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("a");
    let prepared = crate::kpop_engine::KPopEnginePrepared {
        artifacts,
        context: crate::prompt_stratification::WorkflowRenderContext::default(),
        request_text: String::from("r"),
        startup_emit_request: String::from("r"),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        gates: false,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini_max_bash_turns: 1,
        mini_max_http_turns: 1,
        mini_max_bash_execs: 1,
        mini_max_http_retries: 0,
        mini_max_transport_retries: 0,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
    };
    let timing = crate::run_timing::RunTiming::new_arc();
    let p = ExplainKpopPhaseParams {
        shared: &shared,
        workflow: WorkflowCliOptions { force: true },
        prepared: &prepared,
        request_text: "body",
        max_hypotheses: 2,
        outer_iteration: 1,
        phase: EXPLAIN_PHASE_REVIEW,
        run_timing: &timing,
    };
    assert_eq!(p.phase, EXPLAIN_PHASE_REVIEW);
    let backups = SessionDotfileBackups::snapshot(work).expect("snap");
    let result = ExplainKpopPhaseResult {
        chat: String::from("LGTM"),
        backups,
        exp_log_path: work.join("e.md"),
    };
    assert_eq!(result.chat, "LGTM");
}
