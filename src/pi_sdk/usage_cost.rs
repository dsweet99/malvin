use pi::model::{Cost, Message};
use pi::provider::ModelCost;
use pi::sdk::{Config, ModelRegistry};

use super::openrouter_billed_cost;
use super::openrouter_pricing;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct AggregatedCostUsd {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
    pub total: f64,
}

impl AggregatedCostUsd {
    pub(super) const fn is_present(self) -> bool {
        self.total > 0.0
            || self.input > 0.0
            || self.output > 0.0
            || self.cache_read > 0.0
            || self.cache_write > 0.0
    }

    pub(super) fn absorb(&mut self, cost: &Cost) {
        if !cost_is_present(cost) {
            return;
        }
        self.input += cost.input;
        self.output += cost.output;
        self.cache_read += cost.cache_read;
        self.cache_write += cost.cache_write;
        self.total += if cost.total > 0.0 {
            cost.total
        } else {
            cost.input + cost.output + cost.cache_read + cost.cache_write
        };
    }
}

fn cost_is_present(cost: &Cost) -> bool {
    cost.total > 0.0
        || cost.input > 0.0
        || cost.output > 0.0
        || cost.cache_read > 0.0
        || cost.cache_write > 0.0
}

#[allow(clippy::cast_precision_loss)]
fn cost_from_model_rates(rates: &ModelCost, usage: &pi::model::Usage) -> Cost {
    let input = (rates.input / 1_000_000.0) * usage.input as f64;
    let output = (rates.output / 1_000_000.0) * usage.output as f64;
    let cache_read = (rates.cache_read / 1_000_000.0) * usage.cache_read as f64;
    let cache_write = (rates.cache_write / 1_000_000.0) * usage.cache_write as f64;
    Cost {
        input,
        output,
        cache_read,
        cache_write,
        total: input + output + cache_read + cache_write,
    }
}

fn lookup_rates(
    provider: &str,
    model_id: &str,
    registry: Option<&ModelRegistry>,
) -> Option<ModelCost> {
    if let Some(registry) = registry
        && let Some(entry) = registry.find(provider, model_id)
    {
        let rates = &entry.model.cost;
        if rates.input > 0.0
            || rates.output > 0.0
            || rates.cache_read > 0.0
            || rates.cache_write > 0.0
        {
            return Some(rates.clone());
        }
    }
    if provider.eq_ignore_ascii_case("openrouter") {
        return openrouter_pricing::lookup_model_cost(model_id);
    }
    None
}

pub(super) fn aggregate_cost_usd(messages: &[Message]) -> AggregatedCostUsd {
    let mut totals = AggregatedCostUsd::default();
    let registry = pi::auth::AuthStorage::load(Config::auth_path())
        .ok()
        .map(|auth| ModelRegistry::load(&auth, None));
    let mut saw_openrouter_without_reported = false;
    for msg in messages {
        let pi::model::Message::Assistant(assistant) = msg else {
            continue;
        };
        let assistant = assistant.as_ref();
        if cost_is_present(&assistant.usage.cost) {
            totals.absorb(&assistant.usage.cost);
            continue;
        }
        if assistant.provider.eq_ignore_ascii_case("openrouter") {
            saw_openrouter_without_reported = true;
        }
    }
    if totals.is_present() {
        return totals;
    }
    if saw_openrouter_without_reported
        && let Some(billed) = openrouter_billed_cost::fetch_billed_cost_from_generation_ids()
    {
        return billed;
    }
    for msg in messages {
        let pi::model::Message::Assistant(assistant) = msg else {
            continue;
        };
        let assistant = assistant.as_ref();
        if cost_is_present(&assistant.usage.cost) {
            continue;
        }
        let Some(rates) = lookup_rates(&assistant.provider, &assistant.model, registry.as_ref())
        else {
            continue;
        };
        totals.absorb(&cost_from_model_rates(&rates, &assistant.usage));
    }
    totals
}

#[cfg(test)]
#[path = "usage_cost_tests.rs"]
mod usage_cost_tests;
