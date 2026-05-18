#[test]
fn kiss_stringify_orchestrator_units() {
    let _ = stringify!(crate::orchestrator::bug_remediation::run_bug_remediation_gap);
    let _ = stringify!(crate::orchestrator::check_plan::run_check_plan);
    let _ = stringify!(crate::orchestrator::check_plan::run_check_plan_attempt);
    let _ = stringify!(crate::orchestrator::check_plan::read_check_plan_review_file);
    let _ = stringify!(crate::orchestrator::review_loop::run_code_review_phase);
    let _ = stringify!(crate::orchestrator::review_loop::code_review_single_attempt);
    let _ = stringify!(crate::orchestrator::review_loop_helpers::run_concerns_and_check_abort_impl);
    let _ = stringify!(crate::orchestrator::review_loop::CodeReviewAttempt);
    let _ = stringify!(crate::orchestrator::review_loop::CodeReviewAttemptOutcome);
    let _ = stringify!(crate::orchestrator::review_fanout_run::ReviewPromptCoderSession);
    let _ = stringify!(crate::orchestrator::review_fanout_run::run_review_prompt_coder_session);
    let _ = stringify!(crate::orchestrator::review_fanout_run::run_reviewers_spawn_coder_session);
    let _ = stringify!(crate::orchestrator::review_fanout_run::run_review_write_coder_session);
    let _ = stringify!(crate::orchestrator::review_attempt_kernel::read_artifact_review_text);
    let _ = stringify!(crate::orchestrator::review_attempt_kernel::artifact_review_lgtm_after_review_write);
    let _ = stringify!(crate::orchestrator::review_attempt_kernel::ensure_review_prep_after_reviewers_spawn);
    let _ = stringify!(crate::orchestrator::review_attempt_kernel::ensure_artifact_review_after_review_write);
}
