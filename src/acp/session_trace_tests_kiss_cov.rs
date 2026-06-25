//! Kiss identifier refs for `session_trace_tests.rs`.

#[test]
fn kiss_cov_session_trace_test_fns() {
    let _ = super::session_trace_tests::append_prompts_log_do_plain_name_only_writes_do_summary;
    let _ = super::session_trace_tests::append_prompts_log_do_plain_uses_do_stem_like_stdout;
    let _ = super::session_trace_tests::append_prompts_log_uniform_appends_tagged_timestamped_lines;
    let _ = super::session_trace_tests::append_prompts_log_uniform_name_only_writes_one_summary_line;
    let _ = super::session_trace_tests::trace_write_outgoing_prompt_do_preserves_header_user_separator;
    let _ = super::session_trace_tests::trace_write_outgoing_prompt_do_writes_plain_lines_without_tags;
    let _ = super::session_trace_tests::trace_write_tagged_body_writes_prefixed_lines;
}
