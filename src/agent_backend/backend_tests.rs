use super::factory::build_agent_backend;
use super::test_support::test_io;
use crate::model_id::ModelBackend;

#[test]
fn test_io_returns_agent_io_options_with_expected_flags() {
    let io = test_io();
    assert!(io.no_tee);
    assert!(io.raw_output);
    assert!(!io.show_thoughts_on_stdout);
    assert!(!io.emit_stdout_markdown);
    assert!(!io.log_full_outgoing_prompts);
}

#[test]
fn cursor_and_pi_backends_construct_from_prefixed_models() {
    let sdk = crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", test_io(), 1);
    assert!(matches!(sdk.model.backend, ModelBackend::Cursor));
    let pi = {
        let model = crate::model_id::parse_model_id("rpi:openai/gpt-4o").expect("model");
        crate::agent_backend::new_pi(model, test_io())
    };
    assert!(matches!(pi.model.backend, ModelBackend::Pi));
}

#[test]
fn build_agent_backend_selects_pi_for_pi_model() {
    let model = crate::model_id::parse_model_id("rpi:openai/gpt-4o").expect("model");
    let backend = build_agent_backend(model, 3, false).expect("pi backend");
    assert!(matches!(backend.model.backend, ModelBackend::Pi));
}
