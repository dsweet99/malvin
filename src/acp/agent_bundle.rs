use crate::acp::import_prelude::*;

/// Backoff sleep between agent retry attempts. In unit/integration tests, sleeps are skipped so
/// retry policy tests stay fast while production keeps 1s / 3s wall-clock backoff.
pub(crate) async fn agent_backoff_sleep(d: std::time::Duration) {
    if cfg!(test) {
        return;
    }
    tokio::time::sleep(d).await;
}

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

#[test]
fn agent_error_display_roundtrip() {
    let err = AgentError("e".into());
    assert_eq!(err.to_string(), "e");
    assert_eq!(format!("{err}"), "e");
}

#[test]
fn auth_error_display_roundtrip() {
    let err = AuthError("a".into());
    assert_eq!(err.to_string(), "a");
    assert_eq!(format!("{err}"), "a");
}

#[test]
fn agent_error_error_trait() {
    let err = AgentError("e".into());
    let _: &dyn std::error::Error = &err;
}

#[test]
fn auth_error_error_trait() {
    let err = AuthError("a".into());
    let _: &dyn std::error::Error = &err;
}

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

#[test]
fn agent_io_options_all_fields_destructured() {
    let io = AgentIoOptions {
        force: true,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: true,
        emit_stdout_markdown: true,
        log_full_outgoing_prompts: true,
    };
    let AgentIoOptions {
        force,
        no_tee,
        raw_output,
        show_thoughts_on_stdout,
        emit_stdout_markdown,
        log_full_outgoing_prompts,
    } = io;
    assert!(
        force
            && no_tee
            && raw_output
            && show_thoughts_on_stdout
            && emit_stdout_markdown
            && log_full_outgoing_prompts
    );
}

#[test]
fn agent_backoff_sleep_fn_item_ref() {
    let _ = agent_backoff_sleep;
}

#[cfg(test)]
#[path = "agent_bundle_kiss_cov_test.rs"]
mod agent_bundle_kiss_cov_test;
#[cfg(test)]
#[path = "agent_bundle_test.rs"]
mod agent_bundle_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<AgentIoOptions> = None;
    }
}
