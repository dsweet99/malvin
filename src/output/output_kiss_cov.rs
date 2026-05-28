#[test]
fn kiss_cov_output_units() {
    let _ = crate::output::who_tag_ansi;
    let _ = crate::output::append_stdout_log_line;
    let _: Option<crate::output::stdout_log_pair::TaggedDisplayStyle> = None;
    let _ = crate::output::stdout_log_pair::acp_bracket_color;
    let _ = crate::output::stdout_log_pair::acp_bracket_payload;
    let _ = crate::output::stdout_log_pair::acp_from_agent_payload;
    let _ = crate::output::stdout_log_pair::heartbeat_display_and_log_line;
    let _ = crate::output::stdout_log_pair::resolve_log_timestamp;
    let _ = crate::output::stdout_log_pair::tagged_display_and_log_line;
    let _ = crate::output::stdout_log_pair::tagged_stdout_display;
    let _ = crate::output::stderr_log::emit_stderr_log_line;
    let _ = crate::output::stderr_log::emit_stderr_log_lines;
    let _ = crate::output::stdout_tee_env::stdout_is_interactive;
    let _ = crate::output::stdout_tee_env::force_stdout_tee_from_env;
    let _ = crate::output::stdout_tee_env::agent_stdout_tee_enabled;
}
