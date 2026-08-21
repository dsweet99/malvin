use serde_json::json;

use super::map_event::map_codex_stream_events;
use crate::bridge_protocol::BridgeEvent;

#[test]
fn maps_assistant_and_reasoning_deltas() {
    let assistant = map_codex_stream_events(
        "item/agentMessage/delta",
        &json!({ "delta": "hello", "turnId": "t1" }),
    );
    assert!(matches!(
        assistant.as_slice(),
        [BridgeEvent::Assistant { text }] if text == "hello"
    ));
    let thinking = map_codex_stream_events(
        "item/reasoning/textDelta",
        &json!({ "delta": "ponder", "turnId": "t1" }),
    );
    assert!(matches!(
        thinking.as_slice(),
        [BridgeEvent::Thinking { text }] if text == "ponder"
    ));
    let summary = map_codex_stream_events(
        "item/reasoning/summaryTextDelta",
        &json!({ "delta": "plan", "turnId": "t1" }),
    );
    assert!(matches!(
        summary.as_slice(),
        [BridgeEvent::Thinking { text }] if text == "plan"
    ));
    assert!(map_codex_stream_events("item/agentMessage/delta", &json!({ "delta": "" })).is_empty());
    let reasoning_item = map_codex_stream_events(
        "item/completed",
        &json!({
            "item": {
                "id": "r1",
                "type": "reasoning",
                "content": ["think"],
                "summary": ["plan"]
            }
        }),
    );
    assert!(matches!(
        reasoning_item.as_slice(),
        [BridgeEvent::Thinking { text }] if text == "think"
    ));
    let usage = map_codex_stream_events(
        "thread/tokenUsage/updated",
        &json!({
            "tokenUsage": {
                "last": {
                    "inputTokens": 4,
                    "outputTokens": 2,
                    "cachedInputTokens": 1,
                    "cacheWriteInputTokens": 0,
                    "reasoningOutputTokens": 3,
                    "totalTokens": 10
                }
            }
        }),
    );
    match usage.as_slice() {
        [BridgeEvent::Usage { usage }] => {
            assert_eq!(usage["inputTokens"], 4);
            assert_eq!(usage["cacheReadTokens"], 1);
            assert_eq!(usage["reasoningTokens"], 3);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(map_codex_stream_events("thread/tokenUsage/updated", &json!({})).is_empty());
    let objects = map_codex_stream_events(
        "item/completed",
        &json!({
            "item": {
                "id": "r2",
                "type": "reasoning",
                "content": [{"text": "why"}]
            }
        }),
    );
    assert!(matches!(
        objects.as_slice(),
        [BridgeEvent::Thinking { text }] if text == "why"
    ));
    let wrapped = map_codex_stream_events(
        "item/started",
        &json!({
            "item": {
                "id": "c2",
                "type": "commandExecution",
                "command": "/bin/bash -lc cat README.md"
            }
        }),
    );
    assert!(matches!(
        wrapped.as_slice(),
        [BridgeEvent::ToolCall { name, summary, .. }]
            if name.as_deref() == Some("read") && summary.as_deref() == Some("Read cat README.md")
    ));
}

#[test]
fn maps_command_execution_start_and_failure() {
    let start = map_codex_stream_events(
        "item/started",
        &json!({
            "turnId": "t1",
            "item": {
                "id": "c1",
                "type": "commandExecution",
                "command": "ls -la",
                "status": "inProgress"
            }
        }),
    );
    assert!(matches!(
        start.as_slice(),
        [BridgeEvent::ToolCall { phase, name, summary, tool_call_id }]
            if phase == "start"
                && name.as_deref() == Some("shell")
                && summary.as_deref() == Some("Run ls -la")
                && tool_call_id.as_deref() == Some("c1")
    ));
    let failed = map_codex_stream_events(
        "item/completed",
        &json!({
            "item": {
                "id": "c1",
                "type": "commandExecution",
                "command": "false",
                "status": "failed"
            }
        }),
    );
    assert!(matches!(
        failed.as_slice(),
        [BridgeEvent::ToolCall { phase, .. }] if phase == "error"
    ));
}

#[test]
fn maps_file_mcp_and_search_tools() {
    let edit = map_codex_stream_events(
        "item/completed",
        &json!({
            "item": {
                "id": "f1",
                "type": "fileChange",
                "status": "completed",
                "changes": [{ "path": "src/lib.rs", "kind": "update", "diff": "" }]
            }
        }),
    );
    assert!(matches!(
        edit.as_slice(),
        [BridgeEvent::ToolCall { phase, name, summary, .. }]
            if phase == "complete"
                && name.as_deref() == Some("edit")
                && summary.as_deref() == Some("Edit src/lib.rs")
    ));
    let mcp = map_codex_stream_events(
        "item/started",
        &json!({
            "item": {
                "id": "m1",
                "type": "mcpToolCall",
                "server": "docs",
                "tool": "read",
                "arguments": { "path": "README.md" },
                "status": "inProgress"
            }
        }),
    );
    assert!(matches!(
        mcp.as_slice(),
        [BridgeEvent::ToolCall { name, summary, .. }]
            if name.as_deref() == Some("read")
                && summary.as_deref() == Some("Read README.md")
    ));
    let search = map_codex_stream_events(
        "item/started",
        &json!({
            "item": { "id": "s1", "type": "webSearch", "query": "codex app-server" }
        }),
    );
    assert!(matches!(
        search.as_slice(),
        [BridgeEvent::ToolCall { name, summary, .. }]
            if name.as_deref() == Some("webSearch")
                && summary.as_deref() == Some("Search codex app-server")
    ));
}
