use serde_json::{Map, Value};

use super::model_cost::{ModelCost, TokenUsage};
use super::openrouter_pricing::OpenRouterCatalog;
use super::portkey_pricing::PortkeyLive;
use super::pricing_lookup::PricingSource;

const PRICING_SOURCES: [&dyn PricingSource; 2] = [&PortkeyLive, &OpenRouterCatalog];

pub(super) fn lookup_rates(provider: &str, model: &str) -> Option<ModelCost> {
    first_priced(&PRICING_SOURCES, provider, model)
}

fn first_priced(sources: &[&dyn PricingSource], provider: &str, model: &str) -> Option<ModelCost> {
    sources
        .iter()
        .filter_map(|source| source.lookup(provider, model))
        .find(ModelCost::is_priced)
}

fn token_count(obj: &Map<String, Value>, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|key| obj.get(*key).and_then(Value::as_u64))
        .unwrap_or(0)
}

pub(crate) fn apply_portkey_cost_usd(provider: &str, model: &str, usage: &mut Value) {
    super::sdk_usage_fields::normalize_pi_usage(usage);
    if super::sdk_usage_fields::cost_usd_is_positive(usage) {
        return;
    }
    let Some(rates) = lookup_rates(provider, model) else {
        return;
    };
    let Some(obj) = usage.as_object() else {
        return;
    };
    let cost = rates.cost_for(TokenUsage {
        input: token_count(obj, &["inputTokens", "input"]),
        output: token_count(obj, &["outputTokens", "output"]),
        cache_read: token_count(obj, &["cacheReadTokens", "cacheRead", "cache_read"]),
        cache_write: token_count(obj, &["cacheWriteTokens", "cacheWrite", "cache_write"]),
    });
    if let Some(map) = usage.as_object_mut() {
        map.insert(
            "costUsd".into(),
            serde_json::json!({
                "input": cost.input,
                "output": cost.output,
                "cacheRead": cost.cache_read,
                "cacheWrite": cost.cache_write,
                "total": cost.total,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(Option<f64>);

    impl PricingSource for Fixed {
        fn lookup(&self, _provider: &str, _model: &str) -> Option<ModelCost> {
            self.0.map(|input| ModelCost {
                input,
                output: 0.0,
                cache_read: 0.0,
                cache_write: 0.0,
            })
        }
    }

    #[test]
    fn first_priced_takes_the_first_source_with_a_price() {
        let sources: [&dyn PricingSource; 3] = [&Fixed(None), &Fixed(Some(2.0)), &Fixed(Some(9.0))];
        let cost = first_priced(&sources, "p", "m").expect("priced");
        assert!((cost.input - 2.0).abs() < 1e-9);
        assert!(first_priced(&[&Fixed(None)], "p", "m").is_none());
    }

    #[test]
    fn first_priced_skips_negative_and_zero_rate_tables() {
        let sources: [&dyn PricingSource; 3] = [
            &Fixed(Some(-1_000_000.0)),
            &Fixed(Some(0.0)),
            &Fixed(Some(3.0)),
        ];
        let cost = first_priced(&sources, "p", "m").expect("priced");
        assert!((cost.input - 3.0).abs() < 1e-9);
        assert!(first_priced(&[&Fixed(Some(-1.0))], "p", "m").is_none());
    }
}
