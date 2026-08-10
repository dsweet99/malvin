//! ACP trace update kinds (see `concepts.md` §3).
//!
//! Names each ACP-shaped `session/update` kind used in `trace.jsonl` for documentation
//! and typing. Cursor/Prime backends emit standard ACP kinds.

/// One ACP-shaped trace update kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntheticAcpSessionUpdate {
    /// Standard ACP `agent_message_chunk` envelope.
    AgentMessageChunk,
    /// Standard ACP `agent_thought_chunk` envelope.
    AgentThoughtChunk,
    /// Standard ACP `tool_call` envelope.
    ToolCall,
    /// Standard ACP `tool_call_update` envelope.
    ToolCallUpdate,
    /// Non-JSON-RPC `out` trace line (stdout mirror), not a `session/update`.
    OutRaw,
}

impl SyntheticAcpSessionUpdate {
    /// All update kinds in stable concept order.
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
