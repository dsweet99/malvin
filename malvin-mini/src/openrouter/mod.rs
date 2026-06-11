mod client;
mod complete;
mod serde_types;
mod types;

#[cfg(test)]
#[path = "tests.rs"]
mod openrouter_tests;

pub use client::OpenRouterClient;
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};
