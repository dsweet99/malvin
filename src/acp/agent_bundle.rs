
pub(crate) async fn agent_backoff_sleep(d: std::time::Duration) {
    if cfg!(test) || crate::acp::test_no_real_agent_enabled() {
        return;
    }
    tokio::time::sleep(d).await;
}

/// Why an [`AgentError`] occurred, for session-recycle decisions.
///
/// Prefer setting this at the emit site over matching error prose later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentFault {
    /// No typed session implication; teardown may still use legacy message needles.
    #[default]
    Ordinary,
    /// Child, bridge, or stdio transport is dead or unusable.
    SessionDead,
    /// Cursor agent already has an active run.
    CursorBusy,
    /// Cursor SDK auth looks stale.
    StaleAuth,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct AgentError {
    pub message: String,
    pub fault: AgentFault,
}

impl AgentError {
    #[must_use]
    pub fn ordinary(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fault: AgentFault::Ordinary,
        }
    }

    #[must_use]
    pub fn session_dead(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fault: AgentFault::SessionDead,
        }
    }

    #[must_use]
    pub fn cursor_busy(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fault: AgentFault::CursorBusy,
        }
    }

    #[must_use]
    pub fn stale_auth(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            fault: AgentFault::StaleAuth,
        }
    }

    #[must_use]
    pub fn requires_coder_session_teardown(&self) -> bool {
        match self.fault {
            AgentFault::SessionDead | AgentFault::CursorBusy | AgentFault::StaleAuth => true,
            AgentFault::Ordinary => {
                crate::acp::agent_error_requires_coder_session_teardown(&self.message)
            }
        }
    }
}

/// Ordinary [`AgentError`] constructor (value-namespace alias for call-site ergonomics).
#[allow(non_snake_case)]
#[must_use]
pub fn AgentError(message: String) -> AgentError {
    AgentError::ordinary(message)
}

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
    pub log_full_outgoing_prompts: bool,
}

#[cfg(test)]
#[test]
fn agent_error_display_roundtrip() {
    let _ = AgentError;
    let err = AgentError("e".into());
    assert_eq!(err.to_string(), "e");
    assert_eq!(format!("{err}"), "e");
    assert_eq!(err.fault, AgentFault::Ordinary);
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
fn agent_error_session_dead_fault() {
    let err = AgentError::session_dead("bridge write: broken");
    assert_eq!(err.fault, AgentFault::SessionDead);
    assert!(err.requires_coder_session_teardown());
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
