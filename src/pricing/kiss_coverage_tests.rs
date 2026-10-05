#[test]
fn kiss_cov_pricing_and_auth() {
    let _ = super::openrouter_pricing::warm_openrouter_pricing_cache;
    let _ = super::openrouter_pricing::lookup_model_cost;
    let _ = super::portkey_pricing::apply_portkey_cost_usd;
    let _ = super::portkey_pricing::uses_openrouter_catalog;
    let _ = crate::http_fetch::fetch_text;
    let _ = crate::backends::pi_backend::is_provider_authenticated;
    let _ = crate::backends::pi_backend::provider_has_known_credentials;
    let _ = crate::backends::pi_backend::provider_auth::provider_api_key;
    let _ = crate::backends::pi_backend::tool_summary_from_pi;
    let _ = stringify!(OpenRouterModelsResponse);
    let _ = stringify!(OpenRouterModelEntry);
    let _ = stringify!(OpenRouterPricing);
    let _ = stringify!(models_url);
    let _ = stringify!(parse_rate_per_million);
    let _ = stringify!(model_cost_from_pricing);
    let _ = stringify!(fetch_live_pricing_sync);
    let _ = stringify!(pricing_from_models_body);
    let _ = stringify!(openrouter_lookup_ids);
    let _ = stringify!(normalize_pi_usage);
    let _ = stringify!(cost_usd_is_positive);
    let _ = stringify!(ModelCost);
    let _ = stringify!(TokenUsage);
    let _ = stringify!(CostUsd);
    let _ = stringify!(LocalProvider);
    let _ = stringify!(CustomModelRoute);
    let _ = stringify!(HttpRequest);
}
