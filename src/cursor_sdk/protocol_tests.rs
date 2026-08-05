use super::protocol::{decode_event, encode_request, BridgeEvent, BridgeRequest};

#[test]
fn encode_create_uses_camel_case_api_key() {
    let line = encode_request(&BridgeRequest::Create {
        cwd: "/tmp".into(),
        model: "auto".into(),
        api_key: Some("k".into()),
        no_force_policy: Some("fail_fast"),
    })
    .expect("encode");
    assert!(line.contains("\"apiKey\":\"k\""));
    assert!(line.contains("\"noForcePolicy\":\"fail_fast\""));
    assert!(line.contains("\"op\":\"create\""));
}

#[test]
fn decode_run_done_and_fatal() {
    let done = decode_event(
        r#"{"event":"run_done","status":"finished","result":"hi","usage":{"inputTokens":1,"outputTokens":2}}"#,
    )
    .expect("decode");
    match done {
        BridgeEvent::RunDone {
            status, result, ..
        } => {
            assert_eq!(status, "finished");
            assert_eq!(result.as_deref(), Some("hi"));
        }
        other => panic!("unexpected {other:?}"),
    }
    let fatal = decode_event(r#"{"event":"fatal","message":"boom","retryable":true}"#)
        .expect("fatal");
    match fatal {
        BridgeEvent::Fatal { message, retryable } => {
            assert_eq!(message, "boom");
            assert_eq!(retryable, Some(true));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn decode_tool_call_with_enriched_summary() {
    let start = decode_event(
        r#"{"event":"tool_call","phase":"start","name":"shell","summary":"Run ls -ltr","toolCallId":"t1"}"#,
    )
    .expect("decode start");
    match start {
        BridgeEvent::ToolCall {
            phase,
            name,
            summary,
            tool_call_id,
        } => {
            assert_eq!(phase, "start");
            assert_eq!(name.as_deref(), Some("shell"));
            assert_eq!(summary.as_deref(), Some("Run ls -ltr"));
            assert_eq!(tool_call_id.as_deref(), Some("t1"));
        }
        other => panic!("unexpected {other:?}"),
    }
    let err = decode_event(
        r#"{"event":"tool_call","phase":"error","name":"shell","summary":"Run false · exit 1","toolCallId":"t2"}"#,
    )
    .expect("decode error");
    match err {
        BridgeEvent::ToolCall { phase, summary, .. } => {
            assert_eq!(phase, "error");
            assert_eq!(summary.as_deref(), Some("Run false · exit 1"));
        }
        other => panic!("unexpected {other:?}"),
    }
}
