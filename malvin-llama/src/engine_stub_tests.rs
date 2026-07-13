//! Stub engine unit tests (non-Metal targets).

use super::InnerEngine;
use crate::engine::CompleteRequest;

#[test]
fn stub_load_rejects_any_path() {
    let err = InnerEngine::load(std::path::Path::new("/nope.gguf"))
        .err()
        .expect("stub load must fail on non-Metal targets");
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
        .err()
        .expect("stub complete must fail on non-Metal targets");
    assert!(
        err.contains("Apple Silicon"),
        "complete error should name the platform requirement: {err}"
    );
}
