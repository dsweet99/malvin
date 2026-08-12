//! Behavioral and kiss coverage tests for [`super::backend::AgentBackend`].

use super::factory::build_agent_backend;
use super::sdk_client::BridgeKind;
use super::test_support::{shared_opts, test_io};
use crate::cli::WorkflowCliOptions;

#[test]
fn test_io_returns_agent_io_options_with_expected_flags() {
    let io = test_io();
    assert!(io.force);
    assert!(io.no_tee);
    assert!(io.raw_output);
    assert!(!io.show_thoughts_on_stdout);
    assert!(!io.emit_stdout_markdown);
    assert!(!io.log_full_outgoing_prompts);
}

#[test]
fn cursor_and_pi_keep_coder_session_for_process_life() {
    let sdk = crate::agent_backend::agent_backend_from_client(crate::cursor_sdk::cursor_sdk_client_from_raw(
        "cursor:auto",
        test_io(),
        1,
    ));
    assert!(sdk.keeps_coder_session_for_process_life());
    let pi = crate::agent_backend::agent_backend_from_client({
        let model = crate::model_id::parse_model_id("pi:openai/gpt-4o").expect("model");
        crate::agent_backend::SdkClient::new_pi(model, test_io())
    });
    assert!(pi.keeps_coder_session_for_process_life());
}

#[test]
fn build_agent_backend_selects_pi_for_pi_model() {
    let mut shared = shared_opts(false);
    shared.model = crate::model_id::parse_model_id("pi:openai/gpt-4o").expect("model");
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        false,
        "code",
    )
    .expect("pi backend");
    assert!(matches!(backend.kind, BridgeKind::Pi));
}
