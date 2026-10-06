use super::map_event::map_npm_pi_event;
use super::session_turn::{TurnState, feed_mapped_bridge_events};
use super::turn_timeout_tests::cat_npm_pi_session;
use crate::backends::bridge_sdk::tools_in_flight;

#[tokio::test]
async fn issue_43_tool_execution_start_marks_tools_in_flight() {
    let session = cat_npm_pi_session();
    let mut state = TurnState::default();
    let start = serde_json::json!({
        "type": "tool_execution_start",
        "toolCallId": "t1",
        "toolName": "bash",
        "args": {"command": "sleep 1"}
    });
    let events = map_npm_pi_event(&start, &mut state);
    feed_mapped_bridge_events(&session, &events);
    assert!(
        tools_in_flight(&session.stdio.log),
        "issue #43: open tool must set tools_in_flight so busy health keeps the idle window open"
    );
}
