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
    ChatMessage, ChatRole, CompletionResponse, CompletionWithMeta, HttpExchangeMeta, ModelListing,
    OpenRouterClient, ResponseUsage,
};
