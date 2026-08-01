//! Re-export transport errors. Prefer [`crate::llm_transport::TransportError`] at the LlmTransport boundary.

pub use crate::llm_transport::{
    body_indicates_prompt_too_long, is_prompt_too_long_error, TransportError,
};

/// Historical alias for in-tree OpenRouter HTTP helpers; not part of the public LlmTransport API.
pub type OpenRouterError = TransportError;
