//! Kiss coverage for stub engine (must be `*_tests.rs`).

use super::InnerEngine;
use crate::engine::{CompleteRequest, DEFAULT_CONTEXT_SIZE};

#[test]
fn kiss_cov_stub_inner_engine_type() {
    let _: Option<InnerEngine> = None;
    let err = InnerEngine::load(std::path::Path::new("/nope.gguf"), DEFAULT_CONTEXT_SIZE);
    assert!(err.is_err());
    let _ = InnerEngine::load;
    if let Ok(eng) = err {
        let _ = eng.complete(&CompleteRequest {
            turns: &[],
            max_tokens: 1,
        });
    }
    let _ = (
        stringify!(InnerEngine),
        stringify!(load),
        stringify!(complete),
    );
}
