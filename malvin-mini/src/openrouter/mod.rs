mod client;
mod complete;
mod serde_types;
mod types;

pub use client::OpenRouterClient;
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};

#[cfg(test)]
#[path = "openrouter_wire_test.rs"]
mod openrouter_wire_test;
