use pi::model::{AssistantMessage, ContentBlock, Cost, Message, TextContent, Usage};
use pi::provider::ModelCost;

use super::{aggregate_cost_usd, cost_from_model_rates};

fn assistant(provider: &str, model: &str, usage: Usage, cost: Cost) -> Message {
    let mut usage = usage;
    usage.cost = cost;
    Message::assistant(AssistantMessage {
        content: vec![ContentBlock::Text(TextContent::new("hi"))],
        provider: provider.to_string(),
        model: model.to_string(),
        usage,
        ..AssistantMessage::default()
    })
}

#[test]
#[allow(clippy::float_cmp)]
fn aggregate_cost_usd_sums_reported_components() {
    let totals = aggregate_cost_usd(&[assistant(
        "openrouter",
        "x-ai/grok-latest",
        Usage {
            input: 10,
            output: 2,
            ..Usage::default()
        },
        Cost {
            input: 0.01,
            output: 0.02,
            total: 0.03,
            ..Cost::default()
        },
    )]);
    assert_eq!(totals.input, 0.01);
    assert_eq!(totals.output, 0.02);
    assert_eq!(totals.total, 0.03);
}

#[test]
#[allow(clippy::float_cmp)]
fn aggregate_cost_usd_accepts_total_only_reported_cost() {
    let totals = aggregate_cost_usd(&[assistant(
        "openrouter",
        "x-ai/grok-latest",
        Usage {
            input: 10,
            output: 2,
            ..Usage::default()
        },
        Cost {
            total: 0.0042,
            ..Cost::default()
        },
    )]);
    assert_eq!(totals.total, 0.0042);
}

#[test]
#[allow(clippy::float_cmp)]
fn cost_from_model_rates_multiplies_per_million_tokens() {
    let cost = cost_from_model_rates(
        &ModelCost {
            input: 1.0,
            output: 2.0,
            cache_read: 0.1,
            cache_write: 0.2,
        },
        &Usage {
            input: 1_000_000,
            output: 500_000,
            cache_read: 100_000,
            cache_write: 50_000,
            ..Usage::default()
        },
    );
    assert_eq!(cost.input, 1.0);
    assert_eq!(cost.output, 1.0);
    assert_eq!(cost.cache_read, 0.01);
    assert_eq!(cost.cache_write, 0.01);
    assert!((cost.total - 2.02).abs() < 1e-12);
}

#[test]
#[allow(clippy::float_cmp)]
fn aggregate_cost_usd_estimates_from_openrouter_pricing_cache() {
    crate::test_utils::with_isolated_home(|_| {
        let fetched_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cache = serde_json::json!({
            "fetched_at_secs": fetched_at,
            "by_id": {
                "x-ai/grok-latest": {
                    "input": 2.0,
                    "output": 10.0,
                    "cacheRead": 0.0,
                    "cacheWrite": 0.0
                }
            }
        });
        let path = crate::workspace_paths::malvin_user_home_root().join("openrouter-pricing.json");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, cache.to_string()).expect("write cache");

        let totals = aggregate_cost_usd(&[assistant(
            "openrouter",
            "~x-ai/grok-latest",
            Usage {
                input: 1_000_000,
                output: 500_000,
                ..Usage::default()
            },
            Cost::default(),
        )]);
        assert_eq!(totals.input, 2.0);
        assert_eq!(totals.output, 5.0);
        assert_eq!(totals.total, 7.0);
    });
}

#[test]
#[allow(clippy::float_cmp)]
fn aggregate_cost_usd_estimates_without_pi_auth_registry() {
    crate::test_utils::with_isolated_home(|_| {
        let fetched_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cache = serde_json::json!({
            "fetched_at_secs": fetched_at,
            "by_id": {
                "~x-ai/grok-latest": {
                    "input": 1.0,
                    "output": 5.0,
                    "cacheRead": 0.0,
                    "cacheWrite": 0.0
                }
            }
        });
        let path = crate::workspace_paths::malvin_user_home_root().join("openrouter-pricing.json");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(path, cache.to_string()).expect("write cache");

        let totals = aggregate_cost_usd(&[assistant(
            "openrouter",
            "~x-ai/grok-latest",
            Usage {
                input: 1_000_000,
                output: 100_000,
                ..Usage::default()
            },
            Cost::default(),
        )]);
        assert_eq!(totals.input, 1.0);
        assert_eq!(totals.output, 0.5);
        assert_eq!(totals.total, 1.5);
    });
}
