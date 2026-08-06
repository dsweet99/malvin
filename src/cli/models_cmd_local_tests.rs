//! Unit tests for local model listing / prefix filter on `malvin models`.

use super::*;

#[test]
fn models_list_prefix_rejects_download_action() {
    let err = models_list_prefix(&["download".into()]).expect_err("download");
    assert!(err.contains("no longer downloads"), "{err}");
    let err = models_list_prefix(&["download".into(), "mini:local/x".into()]).expect_err("dl");
    assert!(err.contains("no longer downloads"), "{err}");
}

#[test]
fn models_list_prefix_concatenates_words() {
    assert_eq!(models_list_prefix(&[]).expect("empty"), None);
    assert_eq!(
        models_list_prefix(&["prime:".into()]).expect("one").as_deref(),
        Some("prime:")
    );
    assert_eq!(
        models_list_prefix(&["prime:".into(), "open".into()])
            .expect("join")
            .as_deref(),
        Some("prime:open")
    );
}

#[test]
fn section_may_match_prime_open_skips_cursor() {
    assert!(section_may_match(Some("prime:open"), PRIME_PREFIX));
    assert!(!section_may_match(Some("prime:open"), CURSOR_PREFIX));
    assert!(!section_may_match(Some("prime:open"), MINI_OPENROUTER_HEAD));
    assert!(section_may_match(Some("pr"), PRIME_PREFIX));
    assert!(section_may_match(Some("mini:local"), MINI_LOCAL_HEAD));
    assert!(!section_may_match(Some("mini:local"), MINI_OPENROUTER_HEAD));
    assert!(section_may_match(None, CURSOR_PREFIX));
}

#[test]
fn line_matches_prefix_uses_id_before_tab() {
    assert!(line_matches_prefix("prime:openai/gpt\tGPT", Some("prime:open")));
    assert!(!line_matches_prefix("prime:anthropic/x\tX", Some("prime:open")));
    assert!(line_matches_prefix("cursor:auto\tAuto", None));
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
            "Metal builds should list mini:local catalog entries"
        );
        assert!(rows.iter().all(|m| !m.id.is_empty()));
    } else {
        assert!(
            rows.is_empty(),
            "non-Metal builds must omit mini:local from models listing"
        );
    }
}
