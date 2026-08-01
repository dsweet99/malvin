//! OpenRouter HTTP transport (revamp-2 component 1).
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
#[cfg(test)]
#[path = "openrouter/complete_finalize_tests.rs"]
mod complete_finalize_tests;
#[cfg(test)]
#[path = "openrouter/complete_kiss_witness.rs"]
mod complete_kiss_witness;
#[path = "openrouter/complete_parse.rs"]
mod complete_parse;
#[cfg(test)]
#[path = "openrouter/complete_parse_tests.rs"]
mod complete_parse_tests;
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

mod transport_retries;
#[cfg(test)]
#[path = "transport_retries_tests.rs"]
pub(crate) mod transport_retries_tests;

pub use config::OpenRouterConfig;
pub use error::{OpenRouterError, TransportError};
pub use client::OpenRouterClient;
pub use list_models::ModelListing;
pub use http_exchange::{CompletionWithMeta, HttpExchangeMeta};
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};

pub use transport_retries::{
    complete_transport_with_retries, HttpRetryCounters, HttpRetryLimits,
};
