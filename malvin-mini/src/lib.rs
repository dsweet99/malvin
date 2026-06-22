//! OpenRouter HTTP transport for malvin `--mini` (no bash loop, no fence parsing).
#![allow(clippy::multiple_crate_versions)]

mod config;
mod error;
mod openrouter;
mod prompt_shrink;

#[cfg(test)]
mod test_support;

#[cfg(test)]
#[path = "kiss_cov_hub_test.rs"]
mod kiss_cov_hub_test;

pub use config::OpenRouterConfig;
pub use error::OpenRouterError;
pub use openrouter::{ChatMessage, ChatRole, CompletionResponse, OpenRouterClient, ResponseUsage};
