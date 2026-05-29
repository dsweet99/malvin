use crate::acp::PromptRoundHealth;
use serde_json::json;

#[test]
fn detects_upgrade_plan_when_phrase_leads_long_agent_chunk() {
    let text = format!("Upgrade your plan to continue{}", "x".repeat(101));
    let msg = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "agent_message_chunk",
            "content": {"text": text, "type": "text"}
        }}
    });
    let mut h = PromptRoundHealth::default();
    h.record_session_update(&msg);
    assert!(h.upgrade_plan_seen());
}

#[test]
fn append_agent_text_drain_respects_utf8_char_boundary() {
    // 126 ASCII + 2-byte é + 127 ASCII => 255 bytes; drain at 127 splits é.
    let text = format!("{}é{}", "x".repeat(126), "y".repeat(127));
    assert_eq!(text.len(), 255);
    let msg = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "agent_message_chunk",
            "content": {"text": text, "type": "text"}
        }}
    });
    let mut h = PromptRoundHealth::default();
    h.record_session_update(&msg);
}

#[test]
fn detects_upgrade_plan_across_split_agent_chunks() {
    let first = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "agent_message_chunk",
            "content": {"text": "Upgrade your plan", "type": "text"}
        }}
    });
    let second = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "agent_message_chunk",
            "content": {"text": " to continue", "type": "text"}
        }}
    });
    let mut h = PromptRoundHealth::default();
    h.record_session_update(&first);
    assert!(!h.upgrade_plan_seen());
    h.record_session_update(&second);
    assert!(h.upgrade_plan_seen());
}
