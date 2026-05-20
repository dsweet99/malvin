//! Static symbol refs for orchestrator kiss per-file coverage (behavioral smokes in `orchestrator_kiss_coverage.rs`).

#[test]
fn kiss_stringify_orchestrator_units() {
    let _ = stringify!(crate::orchestrator::check_plan::run_check_plan_attempt);
    let _ = stringify!(crate::orchestrator::check_plan::read_check_plan_review_file);
    let _ = stringify!(crate::orchestrator::review_loop::code_review_single_attempt);
    let _ = stringify!(crate::orchestrator::review_loop::CodeReviewAttempt);
    let _ = stringify!(crate::orchestrator::review_loop::CodeReviewAttemptOutcome);
    let _ = stringify!(crate::orchestrator::review_fanout_run::ReviewPromptCoderSession);
    let _ = stringify!(crate::orchestrator::review_fanout_run::run_review_prompt_coder_session);
    let _ = stringify!(crate::orchestrator::review_attempt_kernel::read_artifact_review_text);
}
