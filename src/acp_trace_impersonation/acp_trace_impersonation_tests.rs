use super::SyntheticAcpSessionUpdate;

#[test]
fn synthetic_acp_session_update_all_has_eleven_variants() {
    assert_eq!(SyntheticAcpSessionUpdate::all().len(), 11);
}

#[test]
fn synthetic_acp_session_update_standard_keys() {
    assert_eq!(
        SyntheticAcpSessionUpdate::AgentMessageChunk.session_update_key(),
        Some("agent_message_chunk")
    );
    assert_eq!(
        SyntheticAcpSessionUpdate::AgentThoughtChunk.session_update_key(),
        Some("agent_thought_chunk")
    );
    assert_eq!(
        SyntheticAcpSessionUpdate::ToolCall.session_update_key(),
        Some("tool_call")
    );
    assert_eq!(
        SyntheticAcpSessionUpdate::ToolCallUpdate.session_update_key(),
        Some("tool_call_update")
    );
}

#[test]
fn synthetic_acp_session_update_mini_extensions_have_no_session_key() {
    for variant in [
        SyntheticAcpSessionUpdate::OutRaw,
        SyntheticAcpSessionUpdate::LlmUsage,
        SyntheticAcpSessionUpdate::MiniTerminal,
        SyntheticAcpSessionUpdate::MiniHttpExchange,
        SyntheticAcpSessionUpdate::MiniPromptShrink,
        SyntheticAcpSessionUpdate::MiniPromptShrinkStalled,
        SyntheticAcpSessionUpdate::MiniRetryFork,
    ] {
        assert_eq!(variant.session_update_key(), None);
    }
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
