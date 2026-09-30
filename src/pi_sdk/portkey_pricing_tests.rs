use super::{bare_model_id, host_is_portkey, model_cost_from_body, name_is_portkey_header};

#[test]
fn portkey_host_and_header_detection() {
    assert!(host_is_portkey("https://api.portkey.ai/v1"));
    assert!(host_is_portkey("http://API.PORTKEY.AI/v1"));
    assert!(!host_is_portkey("https://api.openai.com/v1"));
    assert!(!host_is_portkey("https://portkey.example/v1"));
    assert!(name_is_portkey_header("X-Portkey-Api-Key"));
    assert!(!name_is_portkey_header("authorization"));
}

#[test]
fn catalog_slug_keeps_the_model_segment() {
    assert_eq!(bare_model_id("@openai-prod/gpt-4o"), "gpt-4o");
    assert_eq!(bare_model_id("gpt-4o"), "gpt-4o");
}

#[test]
fn pricing_body_converts_cents_per_token_to_dollars_per_million() {
    let body = r#"{"pay_as_you_go":{"request_token":{"price":0.00025},"response_token":{"price":0.001},"cache_read_input_token":{"price":0.000125},"cache_write_input_token":{"price":0}}}"#;
    let cost = model_cost_from_body(body).expect("priced");
    assert!((cost.input - 2.5).abs() < 1e-9);
    assert!((cost.output - 10.0).abs() < 1e-9);
    assert!((cost.cache_read - 1.25).abs() < 1e-9);
    assert!(cost.cache_write.abs() < 1e-9);
}

#[test]
fn missing_pricing_body_is_none() {
    assert!(model_cost_from_body("{}").is_none());
    assert!(model_cost_from_body("not-json").is_none());
}
