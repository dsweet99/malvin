//! In-process llama.cpp completions for malvin `local:` models (Metal on Apple Silicon).
#![allow(clippy::multiple_crate_versions)]

mod chat;
mod engine;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod engine_metal;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod engine_metal_generate;

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
mod engine_stub;

pub use chat::ChatTurn;
pub use engine::{complete, load_engine, CompleteRequest, LocalEngine};

#[cfg(test)]
#[path = "kiss_cov_tests.rs"]
mod kiss_cov_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_turn_role_helpers() {
        let turn = ChatTurn::user("hi");
        assert_eq!(turn.role, "user");
        assert_eq!(turn.content, "hi");
        assert_eq!(ChatTurn::system("s").role, "system");
        assert_eq!(ChatTurn::assistant("a").role, "assistant");
    }
}
