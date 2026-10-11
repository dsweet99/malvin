use std::collections::HashMap;

use super::model_cost::ModelCost;

#[test]
fn cached_openrouter_rates_price_unbilled_usage_and_leave_a_positive_bill() {
    crate::test_support::test_utils::with_isolated_home(|_| {
        super::openrouter_pricing::write_rate_cache_for_test(HashMap::from([(
            "openai/gpt-4o-mini".into(),
            ModelCost {
                input: 0.15,
                output: 0.6,
                cache_read: 0.0,
                cache_write: 0.0,
            },
        )]));
        assert!(super::uses_openrouter_catalog(
            "openrouter",
            "openai/gpt-4o-mini"
        ));
        assert!(!super::uses_openrouter_catalog("anthropic", "claude"));
        let mut usage = serde_json::json!({
            "inputTokens": 1_000_000_u64,
            "outputTokens": 0_u64,
        });
        super::apply_portkey_cost_usd("openrouter", "openai/gpt-4o-mini", &mut usage);
        let total = usage["costUsd"]["total"].as_f64().expect("priced total");
        assert!(
            (total - 0.15).abs() < 1e-9,
            "one million input tokens at $0.15 per million should cost $0.15: {usage}"
        );
        let mut billed = serde_json::json!({
            "inputTokens": 1_000_000_u64,
            "costUsd": {"total": 1.0_f64},
        });
        super::apply_portkey_cost_usd("openrouter", "openai/gpt-4o-mini", &mut billed);
        assert_eq!(
            billed["costUsd"]["total"].as_f64(),
            Some(1.0),
            "an existing positive bill should stay: {billed}"
        );
    });
}

#[test]
fn openai_without_a_key_is_not_authenticated() {
    let _lock = crate::test_support::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path().to_str().expect("utf8").to_string();
    crate::agent_process::with_env("PI_CODING_AGENT_DIR", Some(&dir), || {
        crate::agent_process::with_env("OPENAI_API_KEY", None, || {
            assert!(!crate::backends::pi_backend::is_provider_authenticated(
                "openai"
            ));
            assert!(
                crate::backends::pi_backend::provider_auth::provider_api_key("openai").is_none()
            );
        });
        crate::agent_process::with_env("OPENAI_API_KEY", Some("sk-test"), || {
            assert!(crate::backends::pi_backend::is_provider_authenticated(
                "openai"
            ));
            assert_eq!(
                crate::backends::pi_backend::provider_auth::provider_api_key("openai").as_deref(),
                Some("sk-test")
            );
        });
    });
    assert!(crate::backends::pi_backend::provider_has_known_credentials("openai"));
    assert!(!crate::backends::pi_backend::provider_has_known_credentials("not-a-provider"));
}
