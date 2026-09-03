use pi::model::Cost;

use super::{
    GenerationData, GenerationResponse, cost_from_generation_data,
    fetch_billed_cost_from_generation_ids, urlencoding,
};

#[test]
fn fetch_billed_cost_from_generation_ids_without_capture_returns_none() {
    let _ = fetch_billed_cost_from_generation_ids();
    assert!(fetch_billed_cost_from_generation_ids().is_none());
}

#[test]
fn test_urlencoding_percent_encodes_spaces() {
    assert_eq!(urlencoding("gen-1 hello"), "gen-1%20hello");
}

#[test]
#[allow(clippy::float_cmp)]
fn test_cost_from_generation_data_prefers_usage_field() {
    let cost = cost_from_generation_data(&GenerationData {
        usage: Some(0.0042),
        total_cost: Some(0.99),
        upstream_inference_cost: Some(0.88),
    })
    .expect("cost");
    assert_eq!(cost.total, 0.0042);
}

#[test]
fn generation_response_deserializes_openrouter_shape() {
    let parsed: GenerationResponse = serde_json::from_str(
        r#"{"data":{"usage":0.001,"total_cost":0.001,"upstream_inference_cost":0.0009}}"#,
    )
    .expect("json");
    assert_eq!(parsed.data.usage, Some(0.001));
    let cost: Cost = cost_from_generation_data(&parsed.data).expect("cost");
    assert!((cost.total - 0.001).abs() < f64::EPSILON);
}
