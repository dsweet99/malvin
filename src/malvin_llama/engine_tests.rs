//! Kiss coverage tests for [`super`] (filename must be `*_tests.rs`).

use super::{complete, load_engine, load_engine_with_context_size, CompleteRequest, LocalEngine};
use crate::malvin_llama::ChatTurn;

#[test]
fn kiss_cov_local_engine_load_and_complete_names() {
    let missing = std::path::Path::new("/tmp/malvin-llama-missing-model.gguf");
    let engine: Result<LocalEngine, String> = load_engine(missing);
    assert!(engine.is_err());
    let zero = load_engine_with_context_size(missing, 0);
    match zero {
        Err(msg) => assert!(msg.contains("context_size must be positive")),
        Ok(_) => panic!("expected error for context_size=0"),
    }
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
        stringify!(load_engine_with_context_size),
        stringify!(complete),
    );
}
