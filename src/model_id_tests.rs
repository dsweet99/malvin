use super::*;

#[test]
fn parse_cursor_and_pi() {
    let c = parse_model_id("cursor:auto").expect("cursor");
    assert_eq!(c.backend, ModelBackend::Cursor);
    assert_eq!(c.canonical(), "cursor:auto");
    let pi = parse_model_id("pi:openai/gpt-4o").expect("pi");
    assert!(pi.is_pi());
    assert_eq!(pi.canonical(), "pi:openai/gpt-4o");
    assert_eq!(pi.pi_provider_and_model(), Some(("openai", "gpt-4o")));
    let pi_nested = parse_model_id("pi:openrouter/anthropic/claude-3-haiku").expect("pi nested");
    assert_eq!(
        pi_nested.pi_provider_and_model(),
        Some(("openrouter", "anthropic/claude-3-haiku"))
    );
}

#[test]
fn reject_bare_legacy_and_empty_slug() {
    assert!(parse_model_id("auto").is_err());
    assert!(parse_model_id("cursor:").is_err());
    assert!(parse_model_id("pi:openai").is_err());
    assert!(parse_model_id("prime:openai/gpt-4o")
        .expect_err("legacy prime")
        .contains("prime:"));
    assert!(parse_model_id("mini:openrouter/x")
        .expect_err("legacy mini")
        .contains("mini:"));
    assert!(parse_model_id("openrouter:x")
        .expect_err("legacy")
        .contains("pi:openrouter/"));
    assert!(parse_model_id("local:qwen35_9b_q4")
        .expect_err("legacy")
        .contains("local"));
}

#[test]
fn require_config_and_helpers() {
    assert!(require_config_model("auto").is_err());
    assert_eq!(
        require_config_model("pi:openai/gpt-4o").expect("ok"),
        "pi:openai/gpt-4o"
    );
    assert_eq!(provider_slug("pi:openai/gpt-4o"), "openai/gpt-4o");
    assert!(uses_pi_backend("pi:openai/gpt-4o"));
    assert!(!uses_pi_backend("cursor:auto"));
}

#[test]
fn model_backend_and_parsed_model_debug() {
    let backend = ModelBackend::Cursor;
    let _ = format!("{backend:?}");
    assert_eq!(backend, ModelBackend::Cursor);
    let parsed = ParsedModel {
        backend: ModelBackend::Pi,
        slug: "openai/gpt-4o".into(),
    };
    let clone = parsed.clone();
    assert_eq!(clone, parsed);
    let _ = format!("{parsed:?}");
}
