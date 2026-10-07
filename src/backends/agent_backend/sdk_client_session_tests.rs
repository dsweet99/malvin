use super::*;
use crate::backends::agent_backend::test_support::test_io;
use crate::config::model_id::parse_model_id;
use std::path::PathBuf;

#[test]
fn service_wire_is_codex_only() {
    let cursor = SdkClient::with_max_retries(
        parse_model_id("cursor:auto[service=priority]").expect("cursor"),
        test_io(),
        1,
    );
    assert!(spawn::spawn_service_wire(&cursor).is_none());

    let pi = SdkClient::with_max_retries(
        parse_model_id("pi:openai/gpt-4o[thinking=high]").expect("pi"),
        test_io(),
        1,
    );
    assert!(spawn::spawn_service_wire(&pi).is_none());

    let codex = SdkClient::with_max_retries(
        parse_model_id("codex:gpt-5.6[service=priority]").expect("codex"),
        test_io(),
        1,
    );
    assert_eq!(
        spawn::spawn_service_wire(&codex).as_deref(),
        Some("priority")
    );
}

#[test]
fn force_fresh_coder_agent_drops_resume_id_and_header_satisfaction() {
    let mut client = SdkClient::new(parse_model_id("cursor:auto").expect("model"), test_io());
    client.bind_session_header(
        "header".into(),
        PathBuf::from("/tmp/header.log"),
        "header.md",
    );
    client.header_lifecycle.mark_satisfied_keeping_header();
    client.last_agent_id = Some("old-agent".into());
    malvin_block_on(client.force_fresh_coder_agent()).expect("force fresh");
    assert!(client.last_agent_id.is_none());
    assert!(!crate::backends::agent_backend::session_header_is_satisfied(&client));
    assert!(client.header_lifecycle.pending_header().is_some());
}

fn malvin_block_on<F>(fut: F) -> F::Output
where
    F: std::future::Future,
{
    crate::test_support::test_utils::block_on_test_async(fut)
}
