use super::model_cost::ModelCost;
use super::openrouter_pricing::OpenRouterCatalog;
use super::portkey_pricing::PortkeyLive;

pub(super) trait PricingSource: Sync {
    fn lookup(&self, provider: &str, model: &str) -> Option<ModelCost>;
}

const PRICING_SOURCES: [&dyn PricingSource; 2] = [&PortkeyLive, &OpenRouterCatalog];

pub(super) fn lookup_rates(provider: &str, model: &str) -> Option<ModelCost> {
    first_priced(&PRICING_SOURCES, provider, model)
}

fn first_priced(
    sources: &[&dyn PricingSource],
    provider: &str,
    model: &str,
) -> Option<ModelCost> {
    sources
        .iter()
        .find_map(|source| source.lookup(provider, model))
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
}
