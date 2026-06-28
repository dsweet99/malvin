mod client;
mod complete;
mod http_exchange;
mod serde_types;
mod types;

#[cfg(test)]
#[path = "tests.rs"]
mod openrouter_tests;

#[cfg(test)]
#[path = "prompt_too_long_retry_tests.rs"]
mod prompt_too_long_retry_tests;

#[cfg(test)]
#[path = "fetch_completion_tests.rs"]
mod fetch_completion_tests;

#[cfg(test)]
mod kiss_coverage;

pub use client::OpenRouterClient;
pub use http_exchange::{CompletionWithMeta, HttpExchangeMeta};
pub use types::{
    ChatMessage, ChatRole, CompletionResponse, ResponseUsage,
};
