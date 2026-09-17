use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::models_list::PiModelListing;

pub const LOCAL_LLMS_CONFIG_FILE: &str = "local_llms.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalLlmEntry {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LocalLlmsConfig {
    #[serde(default)]
    pub models: Vec<LocalLlmEntry>,
}

#[must_use]
pub fn local_llms_config_path() -> PathBuf {
    crate::workspace_paths::malvin_user_home_root().join(LOCAL_LLMS_CONFIG_FILE)
}

pub fn load_local_llms_config() -> Result<LocalLlmsConfig, String> {
    let path = local_llms_config_path();
    if !path.is_file() {
        return Ok(LocalLlmsConfig::default());
    }
    let body = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("parse {}: {e}", path.display()))
}

#[cfg(test)]
pub fn save_local_llms_config(cfg: &LocalLlmsConfig) -> Result<(), String> {
    let path = local_llms_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let body =
        serde_json::to_string_pretty(cfg).map_err(|e| format!("serialize local_llms.json: {e}"))?;
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temp, format!("{body}\n")).map_err(|e| format!("write {}: {e}", temp.display()))?;
    fs::rename(&temp, &path).map_err(|e| format!("rename {}: {e}", path.display()))?;
    Ok(())
}

fn configured_allowlist(cfg: &LocalLlmsConfig) -> Option<HashSet<String>> {
    let ids: HashSet<String> = cfg
        .models
        .iter()
        .map(|m| m.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();
    if ids.is_empty() {
        None
    } else {
        Some(ids)
    }
}

pub(crate) fn filter_listings_by_local_llms_config(
    models: Vec<PiModelListing>,
) -> Vec<PiModelListing> {
    let cfg = match load_local_llms_config() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                target: "malvin::pi_sdk",
                error = %e,
                "local_llms.json unreadable; listing without filter"
            );
            return models;
        }
    };
    let Some(allow) = configured_allowlist(&cfg) else {
        return models;
    };
    models
        .into_iter()
        .filter(|m| {
            let provider = m.id.split('/').next().unwrap_or("");
            if !pi::provider_metadata::provider_is_keyless_local(provider) {
                return true;
            }
            allow.contains(&m.id)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_does_not_filter() {
        crate::test_utils::with_isolated_home(|_| {
            let input = vec![
                PiModelListing {
                    id: "ollama/a:latest".into(),
                    name: "a".into(),
                    thinking: None,
                },
                PiModelListing {
                    id: "openai/gpt-4o".into(),
                    name: "gpt-4o".into(),
                    thinking: Some(false),
                },
            ];
            let out = filter_listings_by_local_llms_config(input.clone());
            assert_eq!(out, input);
        });
    }

    #[test]
    fn allowlist_filters_only_keyless_local() {
        crate::test_utils::with_isolated_home(|_| {
            save_local_llms_config(&LocalLlmsConfig {
                models: vec![LocalLlmEntry {
                    id: "ollama/keeper:latest".into(),
                    source: Some("qwen2.5-coder:7b".into()),
                    notes: Some("ft".into()),
                }],
            })
            .expect("save");
            let out = filter_listings_by_local_llms_config(vec![
                PiModelListing {
                    id: "ollama/keeper:latest".into(),
                    name: "keeper".into(),
                    thinking: None,
                },
                PiModelListing {
                    id: "ollama/other:latest".into(),
                    name: "other".into(),
                    thinking: None,
                },
                PiModelListing {
                    id: "openai/gpt-4o".into(),
                    name: "gpt-4o".into(),
                    thinking: Some(false),
                },
            ]);
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].id, "ollama/keeper:latest");
            assert_eq!(out[1].id, "openai/gpt-4o");
            let loaded = load_local_llms_config().expect("load");
            assert_eq!(loaded.models[0].source.as_deref(), Some("qwen2.5-coder:7b"));
        });
    }
}
