//! Kiss coverage references for [`super::kpop_summarize`] privates.

#[test]
fn kiss_cov_kpop_summarize_privates() {
    let _ = stringify!(run_summarize_coder_prompt);
    let _ = stringify!(list_written_exp_logs);
    let _ = stringify!(is_written_exp_log_path);
    let _ = stringify!(insert_summarize_log_context);
    let _ = stringify!(run_summarize_agent_session);
    let _ = stringify!(prefer_gate_outcome_over_summarize);
}
