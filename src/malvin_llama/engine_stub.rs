//! Stub engine for non-Metal targets.

use std::path::Path;

use crate::malvin_llama::engine::CompleteRequest;

#[derive(Debug)]
pub struct InnerEngine;

impl InnerEngine {
    pub fn load(_gguf_path: &Path, _n_ctx: u32) -> Result<Self, String> {
        Err(
            "prime:local/ models require Apple Silicon macOS with Metal (llama.cpp) in this malvin version"
                .into(),
        )
    }

    // Method form matches Metal `InnerEngine::complete`; stub has no instance state.
    #[allow(clippy::unused_self)]
    pub fn complete(&self, _request: &CompleteRequest<'_>) -> Result<String, String> {
        Err("prime:local/ models require Apple Silicon macOS with Metal".into())
    }
}

#[cfg(test)]
#[path = "engine_stub_tests.rs"]
mod engine_stub_tests;
