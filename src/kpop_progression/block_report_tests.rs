use super::block_report::{KpopBlockMissSnapshot, KpopBlockProgressCtx};

#[test]
fn no_progress_error_mentions_hypothesis_counts_and_tool_issues() {
    let snap = KpopBlockMissSnapshot {
        exp_log_path: std::path::PathBuf::from("/tmp/exp.md"),
        hypotheses_before: 0,
        hypotheses_after: 0,
        ctx: KpopBlockProgressCtx {
            steps_needed: 10,
            attempts_so_far: 2,
        },
        tool_health_lines: vec![
            "  - search (Find): Service temporarily unavailable. This may be temporary; try again."
                .to_string(),
        ],
        agent_streamed_kpop_solved: true,
    };
    let msg = snap.format_no_progress_error();
    assert!(msg.contains("made no progress"));
    assert!(msg.contains("0 → 0"));
    assert!(msg.contains("Service temporarily unavailable"));
    assert!(msg.contains("## KPOP_SOLVED"));
}

#[test]
fn catchup_exhausted_error_includes_last_block_need() {
    let snap = KpopBlockMissSnapshot {
        exp_log_path: std::path::PathBuf::from("/tmp/exp.md"),
        hypotheses_before: 0,
        hypotheses_after: 0,
        ctx: KpopBlockProgressCtx {
            steps_needed: 10,
            attempts_so_far: 3,
        },
        tool_health_lines: vec![],
        agent_streamed_kpop_solved: false,
    };
    let msg = snap.format_catchup_exhausted_error();
    assert!(msg.contains("catch-up attempts"));
    assert!(msg.contains("last block still needed: 10"));
}
