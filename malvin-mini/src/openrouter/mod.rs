mod client;
mod complete;
mod serde_types;
mod types;

#[cfg(test)]
#[path = "tests.rs"]
mod openrouter_tests;

#[cfg(test)]
#[path = "prompt_too_long_retry_tests.rs"]
mod prompt_too_long_retry_tests;

#[cfg(test)]
mod kiss_coverage;

pub use client::OpenRouterClient;
pub use types::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};
