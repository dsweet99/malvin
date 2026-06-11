use crate::acp::import_prelude::*;
pub(crate) use tokio::time::sleep as tokio_sleep;

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct AgentError(pub String);

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct AuthError(pub String);

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct AgentIoOptions {
    pub force: bool,
    pub no_tee: bool,
    pub raw_output: bool,
    pub show_thoughts_on_stdout: bool,
    pub emit_stdout_markdown: bool,
    /// When true, echo each outgoing prompt body on stdout and in `prompts.log`; when false, only the `[name...]` line is logged there.
    pub log_full_outgoing_prompts: bool,
}

#[cfg(test)]
#[test]
fn agent_error_display_roundtrip() {
    let _ = AgentError;
    let err = AgentError("e".into());
    assert_eq!(err.to_string(), "e");
    assert_eq!(format!("{err}"), "e");
}

#[cfg(test)]
#[test]
fn auth_error_display_roundtrip() {
    let _ = AuthError;
    let err = AuthError("a".into());
    assert_eq!(err.to_string(), "a");
    assert_eq!(format!("{err}"), "a");
}

#[cfg(test)]
#[test]
fn agent_error_error_trait() {
    let err = AgentError("e".into());
    let _: &dyn std::error::Error = &err;
}

#[cfg(test)]
#[test]
fn auth_error_error_trait() {
    let err = AuthError("a".into());
    let _: &dyn std::error::Error = &err;
}

#[cfg(test)]
#[test]
fn tokio_sleep_reexported() {
    let _ = tokio_sleep;
}

#[cfg(test)]
#[test]
fn agent_io_options_default_fields() {
    let io = AgentIoOptions {
        force: false,
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    };
    assert!(!io.force);
}
