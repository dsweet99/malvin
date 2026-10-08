use super::SyntheticAcpSessionUpdate;
#[test]
fn synthetic_acp_session_update_all_has_five_variants_and_synthetic_acp_session_update_variants_exist()
 {
    {
        assert_eq!(SyntheticAcpSessionUpdate::all().len(), 5);
    }
    {
        let _ = (
            SyntheticAcpSessionUpdate::AgentMessageChunk,
            SyntheticAcpSessionUpdate::AgentThoughtChunk,
            SyntheticAcpSessionUpdate::ToolCall,
            SyntheticAcpSessionUpdate::ToolCallUpdate,
            SyntheticAcpSessionUpdate::OutRaw,
        );
    }
}
