use super::*;

#[test]
fn parse_cursor_and_prime() {
    let c = parse_model_id("cursor:auto").expect("cursor");
    assert_eq!(c.backend, ModelBackend::Cursor);
    assert_eq!(c.canonical(), "cursor:auto");
    let prime = parse_model_id("prime:openai/gpt-5.5").expect("prime");
    assert!(prime.is_prime());
    let prime_local = parse_model_id("prime:local/qwen35_9b_q4").expect("pl");
    assert!(prime_local.is_prime());
    assert!(prime_local.is_prime_local());
    assert_eq!(
        prime_local.local_catalog_slug(),
        Some("qwen35_9b_q4")
    );
    assert!(parse_model_id("prime:local/local/qwen35_9b_q4")
        .expect_err("legacy double local")
        .contains("prime:local/"));
}

#[test]
fn reject_bare_legacy_and_empty_slug() {
    assert!(parse_model_id("auto").is_err());
    assert!(parse_model_id("cursor:").is_err());
    assert!(parse_model_id("prime:openai").is_err());
    assert!(parse_model_id("mini:openrouter/x")
        .expect_err("legacy mini")
        .contains("mini:"));
    assert!(parse_model_id("openrouter:x")
        .expect_err("legacy")
        .contains("prime:openrouter/"));
    assert!(parse_model_id("local:qwen35_9b_q4")
        .expect_err("legacy")
        .contains("prime:local/"));
}

#[test]
fn require_config_and_helpers() {
    assert!(require_config_model("auto").is_err());
    assert_eq!(
        require_config_model("prime:local/qwen35_9b_q4").expect("ok"),
        "prime:local/qwen35_9b_q4"
    );
    assert_eq!(provider_slug("prime:openai/gpt-5.5"), "openai/gpt-5.5");
    assert!(uses_prime_backend("prime:openai/gpt-5.5"));
    assert!(uses_local_backend("prime:local/qwen35_9b_q4"));
    assert!(uses_prime_local_backend("prime:local/qwen35_9b_q4"));
    assert!(!uses_prime_local_backend("prime:openai/gpt-5.5"));
}
