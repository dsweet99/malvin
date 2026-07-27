mod client;
mod complete;
mod list_models;
mod memory_format;
mod models_list_response;
mod http_exchange;
mod provider_error;
mod serde_types;
mod types;

#[cfg(test)]
#[path = "tests.rs"]
mod openrouter_tests;

#[cfg(test)]
#[path = "prompt_too_long_retry_tests.rs"]
mod prompt_too_long_retry_tests;

#[cfg(test)]
#[path = "fetch_completion_tests.rs"]
mod fetch_completion_tests;

#[cfg(test)]
#[path = "list_models_tests.rs"]
mod list_models_tests;

#[cfg(test)]
mod kiss_coverage;

pub use client::OpenRouterClient;
pub use list_models::ModelListing;
pub use http_exchange::{CompletionWithMeta, HttpExchangeMeta};
pub use memory_format::{
    assemble_completion_messages, format_wire_turn, parse_history_response, AssembleInput,
    ParsedTurn, SectionParseError, CHAT_STATE_HISTORY_LABEL, NEW_HISTORY_HEADING,
    PREVIOUS_RESPONSE_LABEL, RESPONSE_HEADING, SECTION_SHAPE_NUDGE,
};
pub use types::{
    ChatMessage, ChatRole, CompletionResponse, ResponseUsage,
};
