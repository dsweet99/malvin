use super::protocol::{
    abort_request, new_session_request, pi_decode_line, pi_encode_request, prompt_request, PiLine,
};

#[test]
fn encode_prompt_and_new_session() {
    let prompt = pi_encode_request(&prompt_request("1", "hello"));
    assert!(prompt.contains("\"type\":\"prompt\""));
    assert!(prompt.contains("\"message\":\"hello\""));
    assert!(prompt.contains("\"id\":\"1\""));
    let ns = pi_encode_request(&new_session_request("2"));
    assert!(ns.contains("\"type\":\"new_session\""));
    let abort = pi_encode_request(&abort_request("3"));
    assert!(abort.contains("\"type\":\"abort\""));
}

#[test]
fn decode_response_and_event() {
    let resp = pi_decode_line(
        r#"{"id":"1","type":"response","command":"prompt","success":true,"data":{}}"#,
    )
    .expect("response");
    match resp {
        PiLine::Response {
            id,
            success,
            command,
            ..
        } => {
            assert_eq!(id, "1");
            assert!(success);
            assert_eq!(command.as_deref(), Some("prompt"));
        }
        PiLine::Event { .. } => panic!("expected response"),
    }
    let ev = pi_decode_line(r#"{"type":"agent_start","sessionId":"abc"}"#).expect("event");
    match ev {
        PiLine::Event { type_name, .. } => assert_eq!(type_name, "agent_start"),
        PiLine::Response { .. } => panic!("expected event"),
    }
}
