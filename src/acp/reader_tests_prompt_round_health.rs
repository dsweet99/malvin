use crate::acp::PromptRoundHealth;
use serde_json::json;

#[test]
fn records_service_unavailable_on_search_tool() {
    let msg = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "kind": "search",
            "title": "Find",
            "status": "completed",
            "rawOutput": {"error": "Service temporarily unavailable. This may be temporary; try again."}
        }}
    });
    let mut h = PromptRoundHealth::default();
    h.record_session_update(&msg);
    assert!(h.has_infra_failure());
    assert!(h.format_lines()[0].contains("Service temporarily unavailable"));
}

#[test]
fn counts_silent_shell_completions() {
    let msg = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    });
    let mut h = PromptRoundHealth::default();
    h.record_session_update(&msg);
    h.record_session_update(&msg);
    assert!(h.has_infra_failure());
}

#[test]
fn detects_streamed_kpop_solved_in_agent_chunk() {
    let msg = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "agent_message_chunk",
            "content": {"text": "## KPOP_SOLVED\nDone.", "type": "text"}
        }}
    });
    let mut h = PromptRoundHealth::default();
    h.record_session_update(&msg);
    assert!(h.agent_streamed_kpop_solved());
}

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
