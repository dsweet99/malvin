use super::*;

#[test]
fn parse_cursor_mini_and_prime() {
    let c = parse_model_id("cursor:auto").expect("cursor");
    assert_eq!(c.backend, ModelBackend::Cursor);
    assert_eq!(c.canonical(), "cursor:auto");
    let o = parse_model_id("mini:openrouter/org/model").expect("or");
    assert!(o.is_openrouter());
    assert_eq!(o.canonical(), "mini:openrouter/org/model");
    let local = parse_model_id("mini:local/qwen35_9b_q4").expect("local");
    assert!(local.is_local());
    let prime = parse_model_id("prime:openai/gpt-5.5").expect("prime");
    assert!(prime.is_prime());
    assert!(!parse_model_id("prime:openrouter/x/y").expect("p").is_openrouter());
}

#[test]
fn reject_bare_legacy_and_empty_slug() {
    assert!(parse_model_id("auto").is_err());
    assert!(parse_model_id("cursor:").is_err());
    assert!(parse_model_id("prime:openai").is_err());
    assert!(parse_model_id("mini:openrouter/").is_err());
    assert!(parse_model_id("openrouter:x").expect_err("legacy").contains("mini:openrouter/"));
    assert!(parse_model_id("local:qwen35_9b_q4").expect_err("legacy").contains("mini:local/"));
}

#[test]
fn require_config_and_helpers() {
    assert!(require_config_model("auto").is_err());
    assert_eq!(
        require_config_model("mini:local/qwen35_9b_q4").expect("ok"),
        "mini:local/qwen35_9b_q4"
    );
    assert_eq!(provider_slug("mini:openrouter/auto"), MINI_DEFAULT_MODEL);
    assert!(uses_mini_backend("mini:openrouter/x"));
    assert!(uses_prime_backend("prime:openai/gpt-5.5"));
    assert!(!uses_openrouter_backend("prime:openrouter/x"));
    assert!(uses_local_backend("mini:local/x"));
}
