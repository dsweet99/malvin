//! OpenRouter HTTP transport for malvin `--mini` (no bash loop, no fence parsing).
#![allow(clippy::multiple_crate_versions)]

mod config;
mod error;
mod openrouter;


#[cfg(test)]
mod test_support;

pub use config::OpenRouterConfig;
pub use error::OpenRouterError;
pub use openrouter::{
    assemble_completion_messages, format_wire_turn, parse_history_response, AssembleInput,
    ChatMessage, ChatRole, CompletionResponse, CompletionWithMeta, HttpExchangeMeta, ModelListing,
    OpenRouterClient, ParsedTurn, ResponseUsage, SectionParseError, CHAT_STATE_HISTORY_LABEL,
    NEW_HISTORY_HEADING, PREVIOUS_RESPONSE_LABEL, RESPONSE_HEADING, SECTION_SHAPE_NUDGE,
};
