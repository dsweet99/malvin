use super::block_report::{KpopBlockMissSnapshot, KpopBlockProgressCtx};

#[test]
fn no_progress_error_mentions_hypothesis_counts_and_tool_issues() {
    let snap = KpopBlockMissSnapshot {
        exp_log_path: std::path::PathBuf::from("/tmp/exp.md"),
        hypotheses_before: 0,
        hypotheses_after: 0,
        ctx: KpopBlockProgressCtx { steps_needed: 10 },
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
    assert!(
        !msg.contains("could not read/write workspace"),
        "must not blame filesystem I/O: {msg}"
    );
}

#[test]
fn no_progress_error_lists_agent_tool_issues_without_filesystem_blame() {
    let snap = KpopBlockMissSnapshot {
        exp_log_path: std::path::PathBuf::from("/tmp/exp.md"),
        hypotheses_before: 0,
        hypotheses_after: 0,
        ctx: KpopBlockProgressCtx { steps_needed: 10 },
        tool_health_lines: vec![
            "  - tool: rg: : IO error for operation on : No such file or directory (os error 2)"
                .to_string(),
        ],
        agent_streamed_kpop_solved: false,
    };
    let msg = snap.format_no_progress_error();
    assert!(msg.contains("ACP tool issues during this prompt"));
    assert!(msg.contains("rg: : IO error"));
    assert!(
        !msg.contains("could not read/write workspace"),
        "must not blame filesystem I/O: {msg}"
    );
}
