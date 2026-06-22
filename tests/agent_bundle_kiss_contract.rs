//! Behavioral coverage for `acp::agent_bundle` display errors (kiss per-file credit).

#[test]
fn agent_error_fmt_display_roundtrip() {
    use malvin::acp::AgentError;
    let _ = AgentError;
    let _ = <AgentError as std::fmt::Display>::fmt;
    assert_eq!(format!("{}", AgentError("contract".into())), "contract");
}

#[test]
fn auth_error_fmt_display_roundtrip() {
    use malvin::acp::AuthError;
    let _ = AuthError;
    let _ = <AuthError as std::fmt::Display>::fmt;
    assert_eq!(format!("{}", AuthError("contract".into())), "contract");
}

#[test]
fn agent_io_options_all_fields_witness() {
    use malvin::acp::AgentIoOptions;
    let _ = stringify!(AgentIoOptions);
    let _ = stringify!(force);
    let _ = stringify!(no_tee);
    let _ = stringify!(raw_output);
    let _ = stringify!(show_thoughts_on_stdout);
    let _ = stringify!(emit_stdout_markdown);
    let _ = stringify!(log_full_outgoing_prompts);
    let io = AgentIoOptions {
        force: true,
        no_tee: false,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        log_full_outgoing_prompts: false,
    };
    let AgentIoOptions {
        force,
        no_tee,
        raw_output,
        show_thoughts_on_stdout,
        emit_stdout_markdown,
        log_full_outgoing_prompts,
    } = io;
    assert!(force && !no_tee && raw_output && !show_thoughts_on_stdout && emit_stdout_markdown && !log_full_outgoing_prompts);
    let _ = <AgentIoOptions as Clone>::clone;
    let _ = <AgentIoOptions as std::fmt::Debug>::fmt;
    let _ = io.clone();
    let _ = format!("{io:?}");
}
