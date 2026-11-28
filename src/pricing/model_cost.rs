use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ModelCost {
    pub(crate) input: f64,
    pub(crate) output: f64,
    pub(crate) cache_read: f64,
    pub(crate) cache_write: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TokenUsage {
    pub(crate) input: u64,
    pub(crate) output: u64,
    pub(crate) cache_read: u64,
    pub(crate) cache_write: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct CostUsd {
    pub(crate) input: f64,
    pub(crate) output: f64,
    pub(crate) cache_read: f64,
    pub(crate) cache_write: f64,
    pub(crate) total: f64,
}

impl ModelCost {
    #[must_use]
    pub(crate) fn is_priced(&self) -> bool {
        let rates = [self.input, self.output, self.cache_read, self.cache_write];
        rates.iter().all(|rate| rate.is_finite() && *rate >= 0.0)
            && rates.iter().any(|rate| *rate > 0.0)
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn cost_for(&self, usage: TokenUsage) -> CostUsd {
        let per = |rate: f64, tokens: u64| rate * tokens as f64 / 1_000_000.0;
        let input = per(self.input, usage.input);
        let output = per(self.output, usage.output);
        let cache_read = per(self.cache_read, usage.cache_read);
        let cache_write = per(self.cache_write, usage.cache_write);
        CostUsd {
            input,
            output,
            cache_read,
            cache_write,
            total: input + output + cache_read + cache_write,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_scales_rates_per_million_tokens() {
        let rates = ModelCost {
            input: 2.0,
            output: 6.0,
            cache_read: 0.5,
            cache_write: 0.0,
        };
        assert!(rates.is_priced());
        let cost = rates.cost_for(TokenUsage {
            input: 1_000_000,
            output: 500_000,
            cache_read: 2_000_000,
            cache_write: 7,
        });
        assert!((cost.total - 6.0).abs() < 1e-12);
        assert!((cost.output - 3.0).abs() < 1e-12);
        assert!(cost.cache_write.abs() < 1e-12);
    }

    #[test]
    fn negative_or_non_finite_rates_are_not_priced() {
        let rates = |input: f64| ModelCost {
            input,
            output: 1.0,
            cache_read: 0.0,
            cache_write: 0.0,
        };
        assert!(!rates(-1.0).is_priced());
        assert!(!rates(f64::NAN).is_priced());
        assert!(!rates(f64::INFINITY).is_priced());
        assert!(rates(0.0).is_priced());
    }

    #[test]
    fn serde_uses_camel_case_cache_fields() {
        let parsed: ModelCost =
            serde_json::from_str(r#"{"input":1,"output":2,"cacheRead":3,"cacheWrite":0}"#)
                .expect("parse");
        assert!((parsed.cache_read - 3.0).abs() < 1e-12);
    }
}
