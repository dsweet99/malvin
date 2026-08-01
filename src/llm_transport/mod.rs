//! Neutral LLM transport interface (Mini → `OpenRouter` or Local).

mod types;
mod completion;
mod error;
mod transport;
pub(crate) mod openrouter;
pub(crate) mod local;

pub use error::TransportError;
pub use error::{body_indicates_prompt_too_long, is_prompt_too_long_error};
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};
pub use completion::{CompletionMeta, CompletionOk};
pub use transport::LlmTransport;
pub use openrouter::OpenRouterTransport;
pub use local::LocalLlmTransport;

/// HTTP exchange metadata retained for adapters/traces (not part of the wire API name).
pub use crate::openrouter_transport::HttpExchangeMeta;

#[cfg(test)]
#[path = "kiss_coverage.rs"]
mod kiss_coverage;

#[cfg(test)]
#[path = "contract_tests.rs"]
mod contract_tests;

