use serde_json::{Map, Value};

const COST_KEYS: [&str; 5] = ["input", "output", "cacheRead", "cacheWrite", "total"];

fn copy_alias(obj: &mut Map<String, Value>, from: &str, to: &str) {
    if obj.contains_key(to) {
        return;
    }
    let Some(value) = obj.get(from).cloned() else {
        return;
    };
    obj.insert(to.to_string(), value);
}

fn cost_object_positive(cost: Option<&Value>) -> bool {
    let Some(obj) = cost.and_then(Value::as_object) else {
        return false;
    };
    COST_KEYS.iter().any(|key| {
        obj.get(*key)
            .and_then(Value::as_f64)
            .is_some_and(|amount| amount > 0.0)
    })
}

pub(super) fn cost_usd_is_positive(usage: &Value) -> bool {
    usage
        .as_object()
        .is_some_and(|obj| cost_object_positive(obj.get("costUsd")))
}

fn promote_cost_usd(obj: &mut Map<String, Value>) {
    if cost_object_positive(obj.get("costUsd")) || !cost_object_positive(obj.get("cost")) {
        return;
    }
    let Some(cost) = obj.get("cost").cloned() else {
        return;
    };
    obj.insert("costUsd".into(), cost);
}

pub(super) fn normalize_pi_usage(usage: &mut Value) {
    let Some(obj) = usage.as_object_mut() else {
        return;
    };
    copy_alias(obj, "input", "inputTokens");
    copy_alias(obj, "output", "outputTokens");
    copy_alias(obj, "cacheRead", "cacheReadTokens");
    copy_alias(obj, "cacheWrite", "cacheWriteTokens");
    copy_alias(obj, "reasoning", "reasoningTokens");
    promote_cost_usd(obj);
}

#[cfg(test)]
mod tests {
    use super::super::portkey_pricing::apply_portkey_cost_usd;

    fn write_openrouter_rate_cache(model_id: &str, input: f64, output: f64) {
        crate::pricing::openrouter_pricing::write_rate_cache_for_test(
            std::collections::HashMap::from([(
                model_id.to_string(),
                crate::pricing::model_cost::ModelCost {
                    input,
                    output,
                    cache_read: 0.0,
                    cache_write: 0.0,
                },
            )]),
        );
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn pi_short_names_estimate_from_openrouter_catalog() {
        crate::test_support::test_utils::with_isolated_home(|_| {
            write_openrouter_rate_cache("x-ai/grok-4.6", 2.0, 6.0);
            let mut usage = serde_json::json!({
                "input": 1_000_000,
                "output": 500_000,
                "cacheRead": 0,
                "cacheWrite": 0,
                "cost": {
                    "input": 0.0,
                    "output": 0.0,
                    "cacheRead": 0.0,
                    "cacheWrite": 0.0,
                    "total": 0.0
                }
            });
            apply_portkey_cost_usd("openrouter", "x-ai/grok-4.6", &mut usage);
            assert_eq!(usage["inputTokens"], 1_000_000);
            assert_eq!(usage["outputTokens"], 500_000);
            assert!((usage["costUsd"]["input"].as_f64().unwrap() - 2.0).abs() < 1e-12);
            assert!((usage["costUsd"]["output"].as_f64().unwrap() - 3.0).abs() < 1e-12);
            assert!((usage["costUsd"]["total"].as_f64().unwrap() - 5.0).abs() < 1e-12);
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn portkey_openrouter_usage_estimates_from_catalog() {
        crate::test_support::test_utils::with_isolated_home(|_| {
            write_openrouter_rate_cache("x-ai/grok-4.6", 2.0, 6.0);
            let dir = tempfile::tempdir().expect("pi dir");
            std::fs::write(
                dir.path().join("models.json"),
                r#"{"providers":{"portkey":{"baseUrl":"https://api.portkey.ai/v1","api":"openai-completions","apiKey":"test","headers":{"x-portkey-provider":"openrouter"},"models":[{"id":"x-ai/grok-4.6","name":"grok"}]}}}"#,
            )
            .expect("models.json");
            let pi_dir = dir.path().to_str().expect("utf8").to_string();
            crate::agent_process::with_env("PI_CODING_AGENT_DIR", Some(&pi_dir), || {
                let mut usage = serde_json::json!({
                    "input": 1_000_000,
                    "output": 1_000_000,
                    "cacheRead": 0,
                    "cost": {
                        "input": 0.0,
                        "output": 0.0,
                        "cacheRead": 0.0,
                        "cacheWrite": 0.0,
                        "total": 0.0
                    }
                });
                apply_portkey_cost_usd("portkey", "x-ai/grok-4.6", &mut usage);
                assert_eq!(usage["inputTokens"], 1_000_000);
                let total = usage["costUsd"]["total"].as_f64().unwrap_or(0.0);
                assert!((total - 8.0).abs() < 1e-9, "costUsd={usage}");
                let mut timing = crate::run_timing::RunTiming::default();
                timing.record_acp_usage_if_present(&usage);
                let recorded = timing.reported_cost_total.unwrap_or(0.0);
                assert!((recorded - 8.0).abs() < 1e-9);
                assert_eq!(timing.tokens_in, Some(1_000_000));
            });
        });
    }

    #[test]
    fn pi_billed_total_is_kept() {
        let mut usage = serde_json::json!({
            "input": 10,
            "output": 2,
            "cost": { "input": 0.0, "output": 0.0, "total": 0.5 }
        });
        apply_portkey_cost_usd("openrouter", "x-ai/grok-4.6", &mut usage);
        assert_eq!(usage["inputTokens"], 10);
        assert_eq!(usage["costUsd"]["total"], 0.5);
    }
}
