//! Neutral completion types used by the local LLM engine and Prime sidecar.

mod types;
mod http_exchange;
mod completion;
mod error;

pub use error::TransportError;
pub use error::{body_indicates_prompt_too_long, is_prompt_too_long_error};
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};
pub use http_exchange::HttpExchangeMeta;
pub use completion::{CompletionMeta, CompletionOk};

#[cfg(test)]
#[path = "kiss_coverage.rs"]
mod kiss_coverage;
