//! ACP trace impersonation (see `concepts.md` §3).
//!
//! Under `--mini`, the in-process bash loop writes synthetic ACP-shaped JSON-RPC
//! `session/update` lines into `trace.jsonl`. This module names each emitted update kind
//! for documentation and typing; emission stays in `acp_trace_shim`.

/// One synthetic trace update kind emitted by the mini ACP trace shim.
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
    /// Mini `miniUsage` extension on an `agent_message_chunk` envelope.
    LlmUsage,
    /// Mini `miniTerminal` extension on an `agent_message_chunk` envelope.
    MiniTerminal,
    /// Mini `miniHttpExchange` extension on an `agent_message_chunk` envelope.
    MiniHttpExchange,
    /// Mini `miniPromptShrink` extension on an `agent_message_chunk` envelope.
    MiniPromptShrink,
    /// Mini `miniPromptShrinkStalled` extension on an `agent_message_chunk` envelope.
    MiniPromptShrinkStalled,
    /// Mini `miniRetryFork` extension on an `agent_message_chunk` envelope.
    MiniRetryFork,
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
            Self::LlmUsage,
            Self::MiniTerminal,
            Self::MiniHttpExchange,
            Self::MiniPromptShrink,
            Self::MiniPromptShrinkStalled,
            Self::MiniRetryFork,
        ]
    }
}

#[cfg(test)]
#[path = "acp_trace_impersonation_tests.rs"]
mod acp_trace_impersonation_tests;
