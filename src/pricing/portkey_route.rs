use serde_json::{Map, Value};

pub(super) struct CustomModelRoute {
    pub(super) base_url: String,
    pub(super) headers: Map<String, Value>,
}

fn object_field<'a>(value: &'a Value, key: &str) -> Option<&'a Map<String, Value>> {
    value.get(key).and_then(Value::as_object)
}

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn provider_model_entry<'a>(
    root: &'a Value,
    provider: &str,
    model_id: &str,
) -> Option<(&'a Value, &'a Value)> {
    let (_, cfg) = object_field(root, "providers")?.iter().find(|(name, _)| {
        crate::local_llm::provider_metadata::provider_ids_match(name, provider)
    })?;
    let model = cfg
        .get("models")?
        .as_array()?
        .iter()
        .find(|m| str_field(m, "id") == Some(model_id))?;
    Some((cfg, model))
}

pub(super) fn route_from_models_json(
    root: &Value,
    provider: &str,
    model_id: &str,
) -> Option<CustomModelRoute> {
    let (cfg, model) = provider_model_entry(root, provider, model_id)?;
    let base_url = str_field(model, "baseUrl")
        .or_else(|| str_field(cfg, "baseUrl"))
        .unwrap_or("")
        .to_string();
    let mut headers = object_field(cfg, "headers").cloned().unwrap_or_default();
    if let Some(extra) = object_field(model, "headers") {
        headers.extend(extra.clone());
    }
    Some(CustomModelRoute { base_url, headers })
}

pub(super) fn custom_model_route(provider: &str, model_id: &str) -> Option<CustomModelRoute> {
    let body =
        std::fs::read_to_string(crate::local_llm::provider_metadata::pi_models_json_path()).ok()?;
    let root: Value = serde_json::from_str(&body).ok()?;
    route_from_models_json(&root, provider, model_id)
}
