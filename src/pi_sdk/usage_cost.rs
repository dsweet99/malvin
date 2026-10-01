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
pub(super) fn cost_from_model_rates(rates: &ModelCost, usage: &pi::model::Usage) -> Cost {
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

pub(super) fn rates_for_provider_model(provider: &str, model_id: &str) -> Option<pi::provider::ModelCost> {
    let registry = load_cost_registry();
    lookup_rates(provider, model_id, registry.as_ref())
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
    let model = super::portkey_pricing::openrouter_catalog_model(provider, model_id)?;
    openrouter_pricing::lookup_model_cost(&model)
}

fn cost_components_present(cost: &Cost) -> bool {
    cost.input > 0.0 || cost.output > 0.0 || cost.cache_read > 0.0 || cost.cache_write > 0.0
}

fn aggregated_components_present(totals: &AggregatedCostUsd) -> bool {
    totals.input > 0.0 || totals.output > 0.0 || totals.cache_read > 0.0 || totals.cache_write > 0.0
}

fn estimate_unsplit_messages(
    messages: &[Message],
    registry: Option<&ModelRegistry>,
) -> AggregatedCostUsd {
    let mut estimated = AggregatedCostUsd::default();
    for msg in messages {
        let pi::model::Message::Assistant(assistant) = msg else {
            continue;
        };
        let assistant = assistant.as_ref();
        if cost_components_present(&assistant.usage.cost) {
            continue;
        }
        let Some(rates) = lookup_rates(&assistant.provider, &assistant.model, registry) else {
            continue;
        };
        estimated.absorb(&cost_from_model_rates(&rates, &assistant.usage));
    }
    estimated
}

fn apply_unsplit_component_estimate(totals: &mut AggregatedCostUsd, estimated: &AggregatedCostUsd) {
    if aggregated_components_present(totals) || !aggregated_components_present(estimated) {
        return;
    }
    totals.input = estimated.input;
    totals.output = estimated.output;
    totals.cache_read = estimated.cache_read;
    totals.cache_write = estimated.cache_write;
    if totals.total <= 0.0 {
        totals.total = estimated.total;
    }
}

fn load_cost_registry() -> Option<ModelRegistry> {
    let auth = pi::auth::AuthStorage::load(Config::auth_path()).ok()?;
    let path = pi::models::default_models_path(&Config::global_dir());
    Some(ModelRegistry::load(&auth, Some(path)))
}

fn absorb_portkey_or_reported(
    totals: &mut AggregatedCostUsd,
    assistant: &pi::model::AssistantMessage,
    saw_openrouter_without_reported: &mut bool,
) {
    if let Some(rates) =
        super::portkey_pricing::rates_for_pi_model(&assistant.provider, &assistant.model)
    {
        totals.absorb(&cost_from_model_rates(&rates, &assistant.usage));
        return;
    }
    if cost_is_present(&assistant.usage.cost) {
        totals.absorb(&assistant.usage.cost);
        return;
    }
    if assistant.provider.eq_ignore_ascii_case("openrouter") {
        *saw_openrouter_without_reported = true;
    }
}

fn absorb_catalog_estimates(
    totals: &mut AggregatedCostUsd,
    messages: &[Message],
    registry: Option<&ModelRegistry>,
) {
    for msg in messages {
        let pi::model::Message::Assistant(assistant) = msg else {
            continue;
        };
        let assistant = assistant.as_ref();
        if cost_is_present(&assistant.usage.cost) {
            continue;
        }
        let Some(rates) = lookup_rates(&assistant.provider, &assistant.model, registry) else {
            continue;
        };
        totals.absorb(&cost_from_model_rates(&rates, &assistant.usage));
    }
}

pub(super) fn aggregate_cost_usd(messages: &[Message]) -> AggregatedCostUsd {
    let mut totals = AggregatedCostUsd::default();
    let registry = load_cost_registry();
    let mut saw_openrouter_without_reported = false;
    for msg in messages {
        let pi::model::Message::Assistant(assistant) = msg else {
            continue;
        };
        absorb_portkey_or_reported(
            &mut totals,
            assistant.as_ref(),
            &mut saw_openrouter_without_reported,
        );
    }
    if !totals.is_present() {
        if saw_openrouter_without_reported
            && let Some(billed) = openrouter_billed_cost::fetch_billed_cost_from_generation_ids()
        {
            totals = billed;
        } else {
            absorb_catalog_estimates(&mut totals, messages, registry.as_ref());
        }
    }
    let estimated = estimate_unsplit_messages(messages, registry.as_ref());
    apply_unsplit_component_estimate(&mut totals, &estimated);
    totals
}

#[cfg(test)]
#[path = "usage_cost_tests.rs"]
mod usage_cost_tests;
