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
    // Path catalogs need `/` between segments; bare concat used to yield mini:openrouteropenai.
    assert_eq!(
        models_list_prefix(&["mini:openrouter".into(), "openai".into()])
            .expect("mini join")
            .as_deref(),
        Some("mini:openrouter/openai")
    );
    assert_eq!(
        models_list_prefix(&["mini:openrouter/".into(), "openai".into()])
            .expect("slash kept")
            .as_deref(),
        Some("mini:openrouter/openai")
    );
    assert_eq!(
        models_list_prefix(&["prime:openai".into(), "gpt".into()])
            .expect("prime path")
            .as_deref(),
        Some("prime:openai/gpt")
    );
    assert_eq!(
        super::models_cmd_filter::join_models_prefix_words(&["cursor:".into(), "auto".into()]),
        "cursor:auto"
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
fn sdk_model_rows_from_stdout_skips_noise_and_injects_auto() {
    use super::test_hooks::sdk_model_rows_from_stdout;

    let rows = sdk_model_rows_from_stdout(
        "\n{\"ok\":true}\n\ncursor:composer-2\tFast\ncursor:default\tDefault\n",
    );
    assert_eq!(rows[0], "cursor:auto");
    assert!(rows.iter().any(|r| r.starts_with("cursor:composer-2")));
    assert!(!rows.iter().any(|r| r.starts_with('{')));
    assert!(!rows.iter().any(String::is_empty));

    let with_auto = sdk_model_rows_from_stdout("cursor:auto\tAuto\ncursor:composer-2\tFast\n");
    assert_eq!(
        with_auto.iter().filter(|r| r.starts_with("cursor:auto")).count(),
        1,
        "must not duplicate cursor:auto when catalog already has it"
    );
}

#[test]
fn models_display_lines_filtered_applies_id_prefix() {
    use super::test_hooks::models_display_lines_filtered;

    let text = "auto - Auto\ncomposer-2 — Fast\n";
    let all = models_display_lines_filtered(text, "cursor:", None).expect("rows");
    assert_eq!(all.len(), 2);
    let filtered =
        models_display_lines_filtered(text, "cursor:", Some("cursor:comp")).expect("filter");
    assert_eq!(filtered, vec!["cursor:composer-2\tFast".to_string()]);
    assert!(models_display_lines_filtered(text, "cursor:", Some("prime:")).is_none());
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
