use serde::Deserialize;

use super::super::model_cost::ModelCost;

#[derive(Debug, Deserialize)]
struct PricingBody {
    pay_as_you_go: Option<PayGo>,
}

#[derive(Debug, Deserialize)]
struct PayGo {
    #[serde(rename = "request_token")]
    request: Option<PricePoint>,
    #[serde(rename = "response_token")]
    response: Option<PricePoint>,
    #[serde(rename = "cache_write_input_token")]
    cache_write: Option<PricePoint>,
    #[serde(rename = "cache_read_input_token")]
    cache_read: Option<PricePoint>,
}

#[derive(Debug, Deserialize)]
struct PricePoint {
    price: f64,
}

fn point_rate(point: Option<&PricePoint>) -> f64 {
    point.map_or(0.0, |p| p.price * 10_000.0)
}

pub(super) fn pricing_model_path(model: &str) -> String {
    model
        .split('/')
        .map(urlencoding)
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn model_cost_from_body(body: &str) -> Option<ModelCost> {
    let parsed: PricingBody = serde_json::from_str(body).ok()?;
    let pay = parsed.pay_as_you_go?;
    let cost = ModelCost {
        input: point_rate(pay.request.as_ref()),
        output: point_rate(pay.response.as_ref()),
        cache_read: point_rate(pay.cache_read.as_ref()),
        cache_write: point_rate(pay.cache_write.as_ref()),
    };
    cost.is_priced().then_some(cost)
}

pub(super) fn urlencoding(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}
