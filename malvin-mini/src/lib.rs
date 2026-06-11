//! OpenRouter HTTP transport for malvin `--mini` (no bash loop, no fence parsing).
#![allow(clippy::multiple_crate_versions)]

mod config;
mod error;
mod openrouter;

pub use config::OpenRouterConfig;
pub use error::OpenRouterError;
pub use openrouter::{ChatMessage, ChatRole, CompletionResponse, OpenRouterClient, ResponseUsage};
