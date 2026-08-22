use super::*;
use crate::agent_backend::test_support::test_io;
use crate::model_id::parse_model_id;

#[test]
fn service_wire_is_codex_only() {
    let cursor = SdkClient::with_max_retries(
        parse_model_id("cursor:auto[service=priority]").expect("cursor"),
        test_io(),
        1,
    );
    assert!(spawn_service_wire(&cursor).is_none());

    let pi = SdkClient::with_max_retries(
        parse_model_id("pi:openai/gpt-4o[thinking=high]").expect("pi"),
        test_io(),
        1,
    );
    assert!(spawn_service_wire(&pi).is_none());

    let codex = SdkClient::with_max_retries(
        parse_model_id("codex:gpt-5.6[service=priority]").expect("codex"),
        test_io(),
        1,
    );
    assert_eq!(spawn_service_wire(&codex).as_deref(), Some("priority"));
}
