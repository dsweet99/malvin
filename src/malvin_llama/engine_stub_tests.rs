//! Stub engine unit tests (non-Metal targets).

use super::InnerEngine;
use crate::malvin_llama::engine::{CompleteRequest, DEFAULT_CONTEXT_SIZE};

#[test]
fn stub_load_rejects_any_path() {
    let err = InnerEngine::load(std::path::Path::new("/nope.gguf"), DEFAULT_CONTEXT_SIZE)
        .expect_err("stub load must fail on non-Metal targets");
    assert!(
        err.contains("Apple Silicon"),
        "load error should name the platform requirement: {err}"
    );
}

#[test]
fn stub_complete_rejects_without_metal() {
    let eng = InnerEngine;
    let err = eng
        .complete(&CompleteRequest {
            turns: &[],
            max_tokens: 1,
        })
        .expect_err("stub complete must fail on non-Metal targets");
    assert!(
        err.contains("Apple Silicon"),
        "complete error should name the platform requirement: {err}"
    );
}
