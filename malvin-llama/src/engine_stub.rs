//! Stub engine for non-Metal targets.

use std::path::Path;

use crate::engine::CompleteRequest;

pub struct InnerEngine;

impl InnerEngine {
    pub fn load(_gguf_path: &Path, _n_ctx: u32) -> Result<Self, String> {
        Err(
            "local: models require Apple Silicon macOS with Metal (llama.cpp) in this malvin version"
                .into(),
        )
    }

    pub fn complete(&self, _request: &CompleteRequest<'_>) -> Result<String, String> {
        Err("local: models require Apple Silicon macOS with Metal".into())
    }
}

#[cfg(test)]
#[path = "engine_stub_tests.rs"]
mod engine_stub_tests;
