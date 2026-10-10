use std::time::Duration;

use super::super::model_cost::ModelCost;
use super::super::pricing_cache_file::PricingCacheFile;
use super::{model_cost_from_body, urlencoding};
use crate::http_fetch::{HttpRequest, fetch_text};

const PRICING_URL: &str = "https://api.portkey.ai/model-configs/pricing";
const CACHE: PricingCacheFile =
    PricingCacheFile::new("portkey-pricing.json", Duration::from_hours(24));

fn pricing_url(provider: &str, model: &str) -> String {
    #[cfg(test)]
    if let Ok(url) = std::env::var("MALVIN_TEST_PORTKEY_PRICING_URL") {
        return url;
    }
    format!(
        "{PRICING_URL}/{}/{}",
        urlencoding(provider),
        super::pricing_model_path(model)
    )
}

#[allow(clippy::missing_const_for_fn)]
fn live_fetch_allowed() -> bool {
    #[cfg(test)]
    {
        std::env::var_os("MALVIN_TEST_PORTKEY_PRICING_URL").is_some()
    }
    #[cfg(not(test))]
    {
        true
    }
}

pub(super) fn lookup_rates(provider: &str, model: &str) -> Option<ModelCost> {
    let key = format!("{provider}/{model}");
    if let Some(cached) = CACHE.fresh_cost(&key) {
        return Some(cached);
    }
    if !live_fetch_allowed() {
        return None;
    }
    let body = fetch_pricing_body(&pricing_url(provider, model))?;
    let cost = model_cost_from_body(&body)?;
    CACHE.insert(&key, cost.clone());
    Some(cost)
}

fn fetch_pricing_body(url: &str) -> Option<String> {
    fetch_text(&HttpRequest::get(url, Duration::from_secs(15)))
}
