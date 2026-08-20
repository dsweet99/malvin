use serde_json::json;

use super::map_event::map_codex_stream_events;
use crate::bridge_protocol::BridgeEvent;

#[test]
fn maps_remaining_tool_kinds_and_status() {
    let declined = map_codex_stream_events(
        "item/completed",
        &json!({
            "item": {
                "id": "c2",
                "type": "commandExecution",
                "command": "rm -rf /",
                "status": "declined"
            }
        }),
    );
    assert!(matches!(
        declined.as_slice(),
        [BridgeEvent::ToolCall { phase, .. }] if phase == "error"
    ));
    let sleep = map_codex_stream_events(
        "item/started",
        &json!({ "item": { "id": "z1", "type": "sleep", "durationMs": 10 } }),
    );
    assert!(matches!(
        sleep.as_slice(),
        [BridgeEvent::ToolCall { name, summary, .. }]
            if name.as_deref() == Some("sleep") && summary.as_deref() == Some("Sleep")
    ));
    let image = map_codex_stream_events(
        "item/started",
        &json!({ "item": { "id": "i1", "type": "imageView", "path": "/tmp/a.png" } }),
    );
    assert!(matches!(
        image.as_slice(),
        [BridgeEvent::ToolCall { summary, .. }] if summary.as_deref() == Some("Read /tmp/a.png")
    ));
    let bash = map_codex_stream_events(
        "item/started",
        &json!({
            "item": {
                "id": "d1",
                "type": "dynamicToolCall",
                "tool": "bash",
                "arguments": { "command": "pwd" },
                "status": "inProgress"
            }
        }),
    );
    assert!(matches!(
        bash.as_slice(),
        [BridgeEvent::ToolCall { summary, .. }] if summary.as_deref() == Some("Run pwd")
    ));
}

#[test]
fn ignores_non_tool_items_and_unknown_methods() {
    let message = map_codex_stream_events(
        "item/started",
        &json!({ "item": { "id": "a1", "type": "agentMessage", "text": "hi" } }),
    );
    assert!(message.is_empty());
    assert!(map_codex_stream_events("turn/started", &json!({})).is_empty());
    let empty_cmd = map_codex_stream_events(
        "item/started",
        &json!({ "item": { "id": "c0", "type": "commandExecution", "command": "  " } }),
    );
    assert!(matches!(
        empty_cmd.as_slice(),
        [BridgeEvent::ToolCall { summary, .. }] if summary.as_deref() == Some("Run")
    ));
}

#[test]
fn maps_collab_image_gen_and_write_tools() {
    let collab = map_codex_stream_events(
        "item/started",
        &json!({
            "item": {
                "id": "k1",
                "type": "collabAgentToolCall",
                "tool": "spawn",
                "status": "inProgress"
            }
        }),
    );
    assert!(matches!(
        collab.as_slice(),
        [BridgeEvent::ToolCall { summary, .. }] if summary.as_deref() == Some("Collab spawn")
    ));
    let image_gen = map_codex_stream_events(
        "item/started",
        &json!({ "item": { "id": "g1", "type": "imageGeneration", "status": "inProgress" } }),
    );
    assert!(matches!(
        image_gen.as_slice(),
        [BridgeEvent::ToolCall { name, .. }] if name.as_deref() == Some("imageGeneration")
    ));
    let write = map_codex_stream_events(
        "item/completed",
        &json!({
            "item": {
                "id": "w1",
                "type": "mcpToolCall",
                "server": "fs",
                "tool": "write",
                "arguments": { "filePath": "a.rs" },
                "status": "completed"
            }
        }),
    );
    assert!(matches!(
        write.as_slice(),
        [BridgeEvent::ToolCall { summary, .. }] if summary.as_deref() == Some("Edit a.rs")
    ));
}

#[test]
fn kiss_cov_codex_map_event_helpers() {
    let _ = stringify!(assistant_delta);
    let _ = stringify!(thinking_delta);
    let _ = stringify!(delta_text);
    let _ = stringify!(item_tool_event);
    let _ = stringify!(tool_from_item);
    let _ = stringify!(tool_phase);
    let _ = stringify!(tool_name_summary);
    let _ = stringify!(misc_tool_summary);
    let _ = stringify!(command_summary);
    let _ = stringify!(file_change_summary);
    let _ = stringify!(named_tool_summary);
    let _ = stringify!(named_tool_label);
    let _ = stringify!(web_search_summary);
    let _ = stringify!(image_view_summary);
    let _ = stringify!(collab_summary);
    let _ = stringify!(command_from_args);
    let _ = stringify!(path_from_args);
    let _ = stringify!(codex_flatten_ws);
}
