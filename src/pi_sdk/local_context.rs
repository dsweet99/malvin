use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::malvin_config_file::load_malvin_config;

#[cfg(test)]
use crate::malvin_config_file::DEFAULT_CONTEXT_SIZE;

pub(crate) const KEYLESS_LOCAL_API_KEY: &str = "local";

pub(crate) const LOCAL_LLM_BASE_URL_ENV: &str = "MALVIN_LOCAL_LLM_BASE_URL";

#[must_use]
pub(crate) fn context_size_for_workdir(work_dir: &Path) -> u32 {
    load_malvin_config(work_dir).context_size.max(1)
}

#[must_use]
pub(crate) fn max_tokens_for_context(context_size: u32) -> u32 {
    (context_size / 4).max(256).min(context_size)
}

fn local_provider_base_url(defaults: &pi::provider_metadata::ProviderRoutingDefaults) -> String {
    if let Ok(value) = std::env::var(LOCAL_LLM_BASE_URL_ENV) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    defaults.base_url.to_string()
}

fn read_models_json(path: &Path) -> Result<Value, String> {
    if !path.is_file() {
        return Ok(json!({ "providers": {} }));
    }
    let body = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn providers_object(root: &mut Value) -> Result<&mut Map<String, Value>, String> {
    root.as_object_mut()
        .ok_or_else(|| "models.json root must be an object".to_string())?
        .entry("providers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| "models.json providers must be an object".to_string())
}

fn provider_object<'a>(
    providers: &'a mut Map<String, Value>,
    provider: &str,
    base_url: &str,
    api: &str,
) -> Result<&'a mut Map<String, Value>, String> {
    providers
        .entry(provider.to_string())
        .or_insert_with(|| {
            json!({
                "baseUrl": base_url,
                "api": api,
                "authHeader": false,
                "models": []
            })
        })
        .as_object_mut()
        .ok_or_else(|| format!("models.json provider `{provider}` must be an object"))
}

fn apply_context_cap(obj: &mut Map<String, Value>, context_size: u32, max_tokens: u32) {
    obj.insert("contextWindow".into(), json!(context_size));
    obj.insert("maxTokens".into(), json!(max_tokens));
}

fn upsert_model_entry(
    models: &mut Vec<Value>,
    model: &str,
    context_size: u32,
    max_tokens: u32,
) -> Result<(), String> {
    if let Some(existing) = models.iter_mut().find(|entry| {
        entry
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == model)
    }) {
        let obj = existing
            .as_object_mut()
            .ok_or_else(|| format!("model `{model}` entry must be an object"))?;
        apply_context_cap(obj, context_size, max_tokens);
        obj.entry("name").or_insert_with(|| json!(model));
        obj.entry("input").or_insert_with(|| json!(["text"]));
        obj.entry("reasoning").or_insert_with(|| json!(false));
        return Ok(());
    }
    models.push(json!({
        "id": model,
        "name": model,
        "input": ["text"],
        "reasoning": false,
        "contextWindow": context_size,
        "maxTokens": max_tokens
    }));
    Ok(())
}

fn clamp_all_provider_models(
    models: &mut [Value],
    context_size: u32,
    max_tokens: u32,
) -> Result<(), String> {
    for (idx, entry) in models.iter_mut().enumerate() {
        let obj = entry
            .as_object_mut()
            .ok_or_else(|| format!("models.json models[{idx}] must be an object"))?;
        let existing = obj
            .get("contextWindow")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| u64::from(context_size));
        if existing > u64::from(context_size) {
            apply_context_cap(obj, context_size, max_tokens);
        }
    }
    Ok(())
}

fn write_models_json(path: &Path, root: &Value) -> Result<(), String> {
    let body =
        serde_json::to_string_pretty(root).map_err(|e| format!("serialize models.json: {e}"))?;
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temp, body).map_err(|e| format!("write {}: {e}", temp.display()))?;
    fs::rename(&temp, path).map_err(|e| format!("rename {}: {e}", path.display()))?;
    Ok(())
}

struct CapModelSpec<'a> {
    provider: &'a str,
    model: &'a str,
    base_url: &'a str,
    api: &'a str,
    context_size: u32,
}

fn apply_capped_model(root: &mut Value, spec: &CapModelSpec<'_>) -> Result<(), String> {
    let providers = providers_object(root)?;
    let provider_entry = provider_object(providers, spec.provider, spec.base_url, spec.api)?;
    provider_entry.insert("baseUrl".into(), json!(spec.base_url));
    provider_entry
        .entry("api")
        .or_insert_with(|| json!(spec.api));
    provider_entry
        .entry("authHeader")
        .or_insert_with(|| json!(false));
    let models = provider_entry
        .entry("models")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| {
            format!(
                "models.json provider `{}` models must be an array",
                spec.provider
            )
        })?;
    let max_tokens = max_tokens_for_context(spec.context_size);
    clamp_all_provider_models(models, spec.context_size, max_tokens)?;
    upsert_model_entry(models, spec.model, spec.context_size, max_tokens)
}

pub(crate) fn ensure_capped_local_model_catalog(
    provider: &str,
    model: &str,
    context_size: u32,
) -> Result<(), String> {
    if !pi::provider_metadata::provider_is_keyless_local(provider) {
        return Ok(());
    }
    let defaults = pi::provider_metadata::provider_routing_defaults(provider)
        .ok_or_else(|| format!("no routing defaults for keyless provider `{provider}`"))?;
    let base_url = local_provider_base_url(&defaults);
    let agent_dir = pi::sdk::Config::global_dir();
    fs::create_dir_all(&agent_dir)
        .map_err(|e| format!("create Pi agent dir {}: {e}", agent_dir.display()))?;
    let path = pi::models::default_models_path(&agent_dir);
    let mut root = read_models_json(&path)?;
    apply_capped_model(
        &mut root,
        &CapModelSpec {
            provider,
            model,
            base_url: &base_url,
            api: defaults.api,
            context_size,
        },
    )?;
    write_models_json(&path, &root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_tokens_scales_with_context() {
        assert_eq!(max_tokens_for_context(DEFAULT_CONTEXT_SIZE), 2048);
        assert_eq!(max_tokens_for_context(256), 256);
        assert_eq!(max_tokens_for_context(1024), 256);
    }

    #[test]
    fn ensure_catalog_writes_capped_ollama_model() {
        crate::test_utils::with_isolated_home(|_| {
            let agent = pi::sdk::Config::global_dir();
            fs::create_dir_all(&agent).expect("agent dir");
            ensure_capped_local_model_catalog("ollama", "malvin-llama32", 4096).expect("ensure");
            let path = pi::models::default_models_path(&agent);
            let root: Value =
                serde_json::from_str(&fs::read_to_string(path).expect("read")).expect("json");
            let model = &root["providers"]["ollama"]["models"][0];
            assert_eq!(model["id"], "malvin-llama32");
            assert_eq!(model["contextWindow"], 4096);
            assert_eq!(model["maxTokens"], 1024);
            assert_eq!(root["providers"]["ollama"]["authHeader"], false);
        });
    }

    #[test]
    fn ensure_catalog_respects_base_url_env() {
        crate::test_utils::with_isolated_home(|_| {
            crate::acp::with_env(
                LOCAL_LLM_BASE_URL_ENV,
                Some("http://host.docker.internal:11434/v1"),
                || {
                    ensure_capped_local_model_catalog("ollama", "malvin-gemma2", 8192)
                        .expect("ensure");
                    let path = pi::models::default_models_path(&pi::sdk::Config::global_dir());
                    let root: Value =
                        serde_json::from_str(&fs::read_to_string(path).expect("read"))
                            .expect("json");
                    assert_eq!(
                        root["providers"]["ollama"]["baseUrl"],
                        "http://host.docker.internal:11434/v1"
                    );
                },
            );
        });
    }

    #[test]
    fn ensure_catalog_skips_non_local_providers() {
        crate::test_utils::with_isolated_home(|_| {
            ensure_capped_local_model_catalog("openai", "gpt-4o", 8192).expect("noop");
            let path = pi::models::default_models_path(&pi::sdk::Config::global_dir());
            assert!(!path.is_file());
        });
    }

    #[test]
    fn ensure_catalog_clamps_sibling_models_with_large_context() {
        crate::test_utils::with_isolated_home(|_| {
            let agent = pi::sdk::Config::global_dir();
            fs::create_dir_all(&agent).expect("agent dir");
            let path = pi::models::default_models_path(&agent);
            fs::write(
                &path,
                r#"{
                  "providers": {
                    "ollama": {
                      "baseUrl": "http://127.0.0.1:11434/v1",
                      "api": "openai-completions",
                      "authHeader": false,
                      "models": [
                        {"id": "stale-big", "name": "stale-big", "contextWindow": 131072, "maxTokens": 8192},
                        {"id": "keep-small", "name": "keep-small", "contextWindow": 2048, "maxTokens": 512}
                      ]
                    }
                  }
                }"#,
            )
            .expect("seed");
            ensure_capped_local_model_catalog("ollama", "new-model", 8192).expect("ensure");
            let root: Value =
                serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
            let models = root["providers"]["ollama"]["models"]
                .as_array()
                .expect("arr");
            let by_id: std::collections::HashMap<&str, &Value> = models
                .iter()
                .filter_map(|m| m.get("id").and_then(Value::as_str).map(|id| (id, m)))
                .collect();
            assert_eq!(by_id["stale-big"]["contextWindow"], 8192);
            assert_eq!(by_id["stale-big"]["maxTokens"], 2048);
            assert_eq!(by_id["keep-small"]["contextWindow"], 2048);
            assert_eq!(by_id["new-model"]["contextWindow"], 8192);
        });
    }

    #[test]
    fn keyless_local_api_key_constant_is_nonempty() {
        assert!(!KEYLESS_LOCAL_API_KEY.trim().is_empty());
    }
}
