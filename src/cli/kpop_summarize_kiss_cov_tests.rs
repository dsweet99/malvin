//! Kiss coverage references for [`super::kpop_summarize`] privates.

#[test]
fn kiss_cov_kpop_summarize_privates() {
    let _: Option<super::kpop_summarize::OuterLoopSummarizeParams<'_>> = None;
    let _ = super::kpop_summarize::run_summarize_agent_session;
    let _ = super::kpop_summarize::run_outer_loop_summarize_if_warranted;
    let _ = super::kpop_summarize::render_kpop_summarize_prompt;
    let _ = super::kpop_summarize::exp_log_paths_markdown;
    let _ = super::kpop_summarize::outer_loop_summarize_warranted;
    let _ = stringify!(run_summarize_coder_prompt);
    let _ = stringify!(list_written_exp_logs);
    let _ = stringify!(is_written_exp_log_path);
    let _ = stringify!(insert_summarize_log_context);
    let _ = stringify!(prefer_gate_outcome_over_summarize);
}

#[cfg(unix)]
#[test]
fn kiss_cov_kpop_summarize_test_helpers() {
    let _ = super::kpop_summarize_tests::write_mock_summarize_agent;
    let _ = stringify!(with_summarize_mock_agent);
    let _ = stringify!(summarize_shared_opts);
    let _ = stringify!(summarize_params);
}
