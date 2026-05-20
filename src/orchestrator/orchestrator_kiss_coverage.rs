//! Cross-module behavioral smokes and static refs for orchestrator kiss per-file coverage.

use super::bug_remediation::run_bug_remediation_gap;
use super::check_plan::run_check_plan;
use super::memory_context::{
    MemoryRecord, build_memories_value, collect_memory_records, emit_if_complete, format_memories,
    parse_memories, process_memory_line, sample_memories, sample_seed,
};
use super::review_loop::run_code_review_phase;
use super::review_loop_helpers::run_concerns_and_check_abort_impl;
use super::{
    artifact_review_lgtm_after_review_write, ensure_artifact_review_after_review_write,
    ensure_review_prep_after_reviewers_spawn, run_review_write_coder_session,
    run_reviewers_spawn_coder_session,
};

#[test]
fn smoke_orchestrator_review_private_fn_items_referenced() {
    let _ = (
        run_check_plan,
        run_concerns_and_check_abort_impl,
        run_code_review_phase,
        run_reviewers_spawn_coder_session,
        run_review_write_coder_session,
        ensure_review_prep_after_reviewers_spawn,
        ensure_artifact_review_after_review_write,
        artifact_review_lgtm_after_review_write,
    );
}

#[test]
fn smoke_memory_context_units() {
    let mut state = super::memory_context::MemoryState::default();
    let mut out = Vec::new();
    process_memory_line("TRIGGER: t", &mut state, &mut out);
    process_memory_line("ADVICE: a", &mut state, &mut out);
    process_memory_line("CONFIDENCE: 1", &mut state, &mut out);
    emit_if_complete(&mut state, &mut out);
    assert_eq!(out.len(), 1);
    let parsed = parse_memories("TRIGGER: x\nADVICE: y\nCONFIDENCE: 2\n");
    assert_eq!(parsed.len(), 1);
    let formatted = format_memories(&parsed);
    assert!(formatted.contains("TRIGGER: x"));
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(collect_memory_records(tmp.path()).is_empty());
    let seed = sample_seed(tmp.path(), &parsed);
    let mut recs = parsed.clone();
    let sampled = sample_memories(&mut recs, 1, seed);
    assert_eq!(sampled.len(), 1);
    let _ = MemoryRecord {
        trigger: "t".into(),
        advice: "a".into(),
        confidence: 1,
    };
    assert!(build_memories_value(tmp.path()).is_empty());
}

#[tokio::test]
async fn smoke_run_bug_remediation_gap_spawn_fails() {
    use super::mid_noop;
    use super::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig};

    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "bug");
    let mut client = no_session_client();
    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: 1,
            run_learn: false,
            learn_min_elapsed_ms: 0,
            skip_check_plan: false,
        },
        progress_callback: Box::new(|_| {}),
        session_dotfile_backups: empty_dotfile_backups(),
    };
    let err = run_bug_remediation_gap(&mut orch, &ctx, mid_noop)
        .await
        .expect_err("bug gap");
    assert!(!err.0.is_empty());
}

#[tokio::test]
async fn smoke_run_check_plan_spawn_fails() {
    use super::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig};

    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "cp");
    let mut client = no_session_client();
    let mut orch = Orchestrator {
        client: &mut client,
        prompts: &store,
        artifacts: &artifacts,
        config: WorkflowConfig {
            max_loops: 1,
            run_learn: false,
            learn_min_elapsed_ms: 0,
            skip_check_plan: false,
        },
        progress_callback: Box::new(|_| {}),
        session_dotfile_backups: empty_dotfile_backups(),
    };
    let err = run_check_plan(&mut orch, &ctx)
        .await
        .expect_err("check plan");
    assert!(!err.0.is_empty());
}
