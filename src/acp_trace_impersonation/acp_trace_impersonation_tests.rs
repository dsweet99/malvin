use super::SyntheticAcpSessionUpdate;

#[test]
fn synthetic_acp_session_update_all_has_five_variants() {
    assert_eq!(SyntheticAcpSessionUpdate::all().len(), 5);
}

#[test]
fn synthetic_acp_session_update_variants_exist() {
    let _ = (
        SyntheticAcpSessionUpdate::AgentMessageChunk,
        SyntheticAcpSessionUpdate::AgentThoughtChunk,
        SyntheticAcpSessionUpdate::ToolCall,
        SyntheticAcpSessionUpdate::ToolCallUpdate,
        SyntheticAcpSessionUpdate::OutRaw,
    );
}
