//! OpenRouter HTTP transport for malvin `--mini` (no bash loop, no fence parsing).
#![allow(clippy::multiple_crate_versions)]
#![allow(
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::items_after_statements,
    clippy::wildcard_enum_match_arm,
    clippy::unnecessary_wraps,
    clippy::redundant_closure_for_method_calls,
    clippy::doc_markdown,
    clippy::derive_partial_eq_without_eq
)]

mod config;
mod error;
mod prompt_shrink;

#[path = "openrouter/client.rs"]
mod client;
#[path = "openrouter/complete.rs"]
mod complete;
#[path = "openrouter/complete_act_detect.rs"]
mod complete_act_detect;
#[path = "openrouter/complete_act_detect_owed.rs"]
mod complete_act_detect_owed;
#[cfg(test)]
#[path = "openrouter/complete_act_detect_tests.rs"]
mod complete_act_detect_tests;
#[path = "openrouter/complete_act_inputs.rs"]
mod complete_act_inputs;
#[cfg(test)]
#[path = "openrouter/complete_act_inputs_tests.rs"]
mod complete_act_inputs_tests;
#[path = "openrouter/complete_fail_epoch.rs"]
mod complete_fail_epoch;
#[cfg(test)]
#[path = "openrouter/complete_finalize_tests.rs"]
mod complete_finalize_tests;
#[cfg(test)]
#[path = "openrouter/complete_kiss_witness.rs"]
mod complete_kiss_witness;
#[path = "openrouter/complete_local_retry.rs"]
mod complete_local_retry;
#[cfg(test)]
#[path = "openrouter/complete_local_retry_act_pressure_tests.rs"]
mod complete_local_retry_act_pressure_tests;
#[cfg(test)]
#[path = "openrouter/complete_local_retry_act_tests.rs"]
mod complete_local_retry_act_tests;
#[path = "openrouter/complete_local_retry_pressure.rs"]
mod complete_local_retry_pressure;
#[cfg(test)]
#[path = "openrouter/complete_local_retry_req_tests.rs"]
mod complete_local_retry_req_tests;
#[cfg(test)]
#[path = "openrouter/complete_local_retry_tests.rs"]
mod complete_local_retry_tests;
#[path = "openrouter/complete_marker_shape.rs"]
mod complete_marker_shape;
#[path = "openrouter/complete_parse.rs"]
mod complete_parse;
#[cfg(test)]
#[path = "openrouter/complete_parse_tests.rs"]
mod complete_parse_tests;
#[path = "openrouter/complete_prompt_shape.rs"]
mod complete_prompt_shape;
#[cfg(test)]
#[path = "openrouter/complete_prompt_shape_tests.rs"]
mod complete_prompt_shape_tests;
#[path = "openrouter/complete_prompt_shrink.rs"]
mod complete_prompt_shrink;
#[path = "openrouter/complete_requirements_path.rs"]
mod complete_requirements_path;
#[cfg(test)]
#[path = "openrouter/complete_requirements_path_tests.rs"]
mod complete_requirements_path_tests;
#[path = "openrouter/complete_requirements_shape.rs"]
mod complete_requirements_shape;
#[path = "openrouter/complete_section_shape.rs"]
mod complete_section_shape;
#[cfg(test)]
#[path = "openrouter/fetch_completion_tests.rs"]
mod fetch_completion_tests;
#[path = "openrouter/http_exchange.rs"]
mod http_exchange;
#[cfg(test)]
#[path = "openrouter/kiss_coverage.rs"]
mod kiss_coverage;
#[path = "openrouter/list_models.rs"]
mod list_models;
#[cfg(test)]
#[path = "openrouter/list_models_tests.rs"]
mod list_models_tests;
#[path = "openrouter/memory_format.rs"]
mod memory_format;
#[cfg(test)]
#[path = "openrouter/memory_format_tests.rs"]
mod memory_format_tests;
#[path = "openrouter/models_list_response.rs"]
mod models_list_response;
#[cfg(test)]
#[path = "openrouter/models_list_response_tests.rs"]
mod models_list_response_tests;
#[cfg(test)]
#[path = "openrouter/prompt_too_long_retry_tests.rs"]
mod prompt_too_long_retry_tests;
#[path = "openrouter/provider_error.rs"]
mod provider_error;
#[cfg(test)]
#[path = "openrouter/provider_error_tests.rs"]
mod provider_error_tests;
#[path = "openrouter/serde_types.rs"]
mod serde_types;
#[cfg(test)]
#[path = "openrouter/tests.rs"]
mod openrouter_tests;
#[path = "openrouter/types.rs"]
mod types;

#[cfg(test)]
mod test_support;

pub use config::OpenRouterConfig;
pub use error::OpenRouterError;
pub use client::OpenRouterClient;
pub use list_models::ModelListing;
pub use http_exchange::{CompletionWithMeta, HttpExchangeMeta};
pub use memory_format::{
    assemble_completion_messages, format_wire_turn, parse_history_response, AssembleInput,
    ParsedTurn, SectionParseError, CHAT_STATE_HISTORY_LABEL, NEW_HISTORY_HEADING,
    PREVIOUS_RESPONSE_LABEL, RESPONSE_HEADING, SECTION_SHAPE_NUDGE,
};
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};
