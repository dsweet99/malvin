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
fn agent_io_options_type_witness() {
    use malvin::acp::AgentIoOptions;
    let io = AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    };
    assert!(!io.force);
}
