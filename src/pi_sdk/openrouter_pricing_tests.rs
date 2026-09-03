use std::collections::HashMap;
use std::fs;

use pi::provider::ModelCost;

use super::super::cache_clock::{cache_fetched_at_is_fresh, unix_now_secs};
use super::{
    CACHE_TTL, cache_path, fetch_live_pricing_sync, load_cache, lookup_model_cost,
    openrouter_lookup_ids, parse_rate_per_million, save_cache, warm_openrouter_pricing_cache,
};

#[test]
#[allow(clippy::float_cmp)]
fn parse_rate_per_million_scales_token_price() {
    assert_eq!(parse_rate_per_million("0.000001"), Some(1.0));
}

#[test]
fn cache_freshness_matches_daily_ttl() {
    assert!(cache_fetched_at_is_fresh(unix_now_secs(), CACHE_TTL));
    let stale = unix_now_secs().saturating_sub(CACHE_TTL.as_secs() + 1);
    assert!(!cache_fetched_at_is_fresh(stale, CACHE_TTL));
}

#[test]
#[allow(clippy::float_cmp)]
fn fetch_live_pricing_parses_mock_models_response() {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let body = r#"{"data":[{"id":"x-ai/grok-latest","pricing":{"prompt":"0.000002","completion":"0.000010","input_cache_read":"0.000001"}}]}"#;
    std::thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            let _ = socket.read(&mut [0_u8; 512]);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = socket.write_all(response.as_bytes());
        }
    });
    let url = format!("http://{addr}/models");
    let costs = fetch_live_pricing_sync("test-key", &url).expect("costs");
    let cost = costs.get("x-ai/grok-latest").expect("model");
    assert_eq!(cost.input, 2.0);
    assert_eq!(cost.output, 10.0);
    assert_eq!(cost.cache_read, 1.0);
}

#[test]
#[allow(clippy::float_cmp)]
fn model_cost_from_pricing_parses_cache_tiers() {
    use super::OpenRouterPricing;
    use super::model_cost_from_pricing;

    let cost = model_cost_from_pricing(&OpenRouterPricing {
        prompt: "0.000002".into(),
        completion: "0.000003".into(),
        input_cache_read: Some("0.000001".into()),
        input_cache_write: Some("0.000004".into()),
    })
    .expect("cost");
    assert_eq!(cost.input, 2.0);
    assert_eq!(cost.output, 3.0);
    assert_eq!(cost.cache_read, 1.0);
    assert_eq!(cost.cache_write, 4.0);
}

#[test]
fn openrouter_lookup_ids_expands_latest_alias() {
    let ids = openrouter_lookup_ids("~x-ai/grok-latest");
    assert!(ids.iter().any(|id| id == "x-ai/grok-latest"));
}

#[test]
fn save_and_load_cache_round_trip() {
    crate::test_utils::with_isolated_home(|_| {
        let mut by_id = HashMap::new();
        by_id.insert(
            "vendor/model".into(),
            ModelCost {
                input: 1.0,
                output: 2.0,
                cache_read: 0.5,
                cache_write: 0.25,
            },
        );
        save_cache(by_id.clone());
        let loaded = load_cache().expect("cache");
        assert!(cache_fetched_at_is_fresh(loaded.fetched_at_secs, CACHE_TTL));
        assert_eq!(loaded.by_id, by_id);
    });
}

#[test]
#[allow(clippy::float_cmp)]
fn lookup_model_cost_reads_cached_openrouter_rates() {
    crate::test_utils::with_isolated_home(|_| {
        save_cache(HashMap::from([(
            "x-ai/grok-latest".into(),
            ModelCost {
                input: 3.0,
                output: 15.0,
                cache_read: 0.75,
                cache_write: 0.0,
            },
        )]));
        let cost = lookup_model_cost("~x-ai/grok-latest").expect("cost");
        assert_eq!(cost.input, 3.0);
        assert_eq!(cost.output, 15.0);
    });
}

#[test]
fn warm_openrouter_pricing_cache_skips_when_cache_is_fresh() {
    crate::test_utils::with_isolated_home(|_| {
        save_cache(HashMap::from([(
            "openai/gpt-4o-mini".into(),
            ModelCost {
                input: 0.15,
                output: 0.6,
                cache_read: 0.075,
                cache_write: 0.0,
            },
        )]));
        let path = cache_path();
        let before = fs::read_to_string(&path).expect("cache");
        warm_openrouter_pricing_cache(false);
        let after = fs::read_to_string(path).expect("cache");
        assert_eq!(before, after);
    });
}

#[test]
fn warm_openrouter_pricing_cache_noops_without_api_key() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENROUTER_API_KEY", None, || {
            warm_openrouter_pricing_cache(true);
            assert!(!cache_path().exists());
        });
    });
}
