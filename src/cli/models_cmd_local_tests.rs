//! Unit tests for local model listing / download actions on `malvin models`.

use super::*;

#[test]
fn run_models_action_rejects_bad_usage() {
    let err = run_models_action(&["download".into()]).expect_err("usage");
    assert!(err.contains("usage"));
    let err = run_models_action(&["nope".into()]).expect_err("unknown");
    assert!(err.contains("unknown"));
}

#[test]
fn models_args_default_has_empty_words() {
    let args = ModelsArgs::default();
    assert!(args.words.is_empty());
}

#[test]
fn local_listings_omitted_without_metal() {
    let rows = local_model_listings();
    if crate::local_llm::local_backend_supported() {
        assert!(
            !rows.is_empty(),
            "Metal builds should list local: catalog entries"
        );
        assert!(rows.iter().all(|m| !m.id.is_empty()));
    } else {
        assert!(
            rows.is_empty(),
            "non-Metal builds must omit local: from models listing"
        );
    }
}
