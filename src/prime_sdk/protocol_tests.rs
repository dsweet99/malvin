use super::protocol::{prime_decode_event, prime_encode_request, PrimeBridgeEvent, PrimeBridgeRequest};

#[test]
fn encode_create_send_cancel_close() {
    let line = prime_encode_request(&PrimeBridgeRequest::Create {
        cwd: "/tmp".into(),
        model: "openai/gpt-5.5".into(),
        api_key: None,
        no_force_policy: None,
    })
    .expect("encode");
    assert!(line.contains("\"op\":\"create\""));
    assert!(!line.contains("resume"));
    let send = prime_encode_request(&PrimeBridgeRequest::Send {
        prompt: "hi".into(),
    })
    .expect("send");
    assert!(send.contains("\"op\":\"send\""));
    let _ = prime_encode_request(&PrimeBridgeRequest::Cancel {}).expect("cancel");
    let _ = prime_encode_request(&PrimeBridgeRequest::Close {}).expect("close");
}

#[test]
fn decode_assistant_and_run_done() {
    let ev = prime_decode_event(r#"{"event":"assistant","text":"hi"}"#).expect("ok");
    assert!(matches!(ev, PrimeBridgeEvent::Assistant { text } if text == "hi"));
    let done = prime_decode_event(r#"{"event":"run_done","status":"ok"}"#).expect("done");
    assert!(matches!(done, PrimeBridgeEvent::RunDone { status, .. } if status == "ok"));
    let _ = PrimeBridgeEvent::Thinking {
        text: "t".into(),
    };
    let _ = PrimeBridgeEvent::ToolCall {
        phase: "start".into(),
        name: None,
        summary: None,
        tool_call_id: None,
    };
    let _ = PrimeBridgeEvent::Step { kind: None };
    let _ = PrimeBridgeEvent::Usage {
        usage: serde_json::json!({}),
    };
    let _ = PrimeBridgeEvent::Fatal {
        message: "x".into(),
        retryable: None,
    };
    let _ = PrimeBridgeEvent::Ok { agent_id: None };
    let _ = PrimeBridgeEvent::Unknown;
}
