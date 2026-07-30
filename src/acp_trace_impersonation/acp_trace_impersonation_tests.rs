use super::SyntheticAcpSessionUpdate;

#[test]
fn synthetic_acp_session_update_all_has_eleven_variants() {
    assert_eq!(SyntheticAcpSessionUpdate::all().len(), 11);
}

#[test]
fn synthetic_acp_session_update_variants_exist() {
    let _ = (
        SyntheticAcpSessionUpdate::AgentMessageChunk,
        SyntheticAcpSessionUpdate::AgentThoughtChunk,
        SyntheticAcpSessionUpdate::ToolCall,
        SyntheticAcpSessionUpdate::ToolCallUpdate,
        SyntheticAcpSessionUpdate::OutRaw,
        SyntheticAcpSessionUpdate::LlmUsage,
        SyntheticAcpSessionUpdate::MiniTerminal,
        SyntheticAcpSessionUpdate::MiniHttpExchange,
        SyntheticAcpSessionUpdate::MiniPromptShrink,
        SyntheticAcpSessionUpdate::MiniPromptShrinkStalled,
        SyntheticAcpSessionUpdate::MiniRetryFork,
    );
}
