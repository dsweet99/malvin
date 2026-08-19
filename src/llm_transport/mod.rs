mod completion;
mod error;
mod http_exchange;
mod types;

pub use completion::{CompletionMeta, CompletionOk};
pub use error::TransportError;
pub use error::{body_indicates_prompt_too_long, is_prompt_too_long_error};
pub use http_exchange::HttpExchangeMeta;
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};

#[cfg(test)]
#[path = "kiss_coverage.rs"]
mod kiss_coverage;
