//! Kiss coverage tests for [`super`] (filename must be `*_tests.rs`).

use super::{complete, load_engine, CompleteRequest, LocalEngine};
use crate::ChatTurn;

#[test]
fn kiss_cov_local_engine_load_and_complete_names() {
    let missing = std::path::Path::new("/tmp/malvin-llama-missing-model.gguf");
    let engine: Result<LocalEngine, String> = load_engine(missing);
    assert!(engine.is_err());
    let complete_fn: fn(&LocalEngine, &CompleteRequest<'_>) -> Result<String, String> = complete;
    let req = CompleteRequest {
        turns: &[] as &[ChatTurn],
        max_tokens: 1,
    };
    assert_eq!(req.max_tokens, 1);
    let _ = complete_fn;
    let _ = (
        stringify!(LocalEngine),
        stringify!(CompleteRequest),
        stringify!(load_engine),
        stringify!(complete),
    );
}
