
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntheticAcpSessionUpdate {
    AgentMessageChunk,
    AgentThoughtChunk,
    ToolCall,
    ToolCallUpdate,
    OutRaw,
}

impl SyntheticAcpSessionUpdate {
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::AgentMessageChunk,
            Self::AgentThoughtChunk,
            Self::ToolCall,
            Self::ToolCallUpdate,
            Self::OutRaw,
        ]
    }
}

#[cfg(test)]
#[path = "acp_trace_impersonation_tests.rs"]
mod acp_trace_impersonation_tests;
