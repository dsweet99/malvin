use std::collections::{HashMap, HashSet};

use pi::sdk::ModelRegistry;

use super::models_list::PiModelListing;

pub(crate) fn merge_registry_with_live(
    registry: &ModelRegistry,
    live_by_provider: &HashMap<String, Vec<String>>,
) -> Vec<PiModelListing> {
    let static_by_key = static_registry_lookup(registry);
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    append_live_models(&mut out, &mut seen, &static_by_key, live_by_provider);
    append_static_models_without_live(&mut out, &mut seen, registry, live_by_provider);
    out
}

fn static_registry_lookup(
    registry: &ModelRegistry,
) -> HashMap<(String, String), &pi::models::ModelEntry> {
    registry
        .models()
        .iter()
        .map(|entry| {
            (
                (entry.model.provider.clone(), entry.model.id.clone()),
                entry,
            )
        })
        .collect()
}

fn append_live_models(
    out: &mut Vec<PiModelListing>,
    seen: &mut HashSet<String>,
    static_by_key: &HashMap<(String, String), &pi::models::ModelEntry>,
    live_by_provider: &HashMap<String, Vec<String>>,
) {
    for (provider, ids) in live_by_provider {
        for id in ids {
            let full_id = format!("{provider}/{id}");
            if !seen.insert(full_id.clone()) {
                continue;
            }
            let (name, thinking) = static_by_key
                .get(&(provider.clone(), id.clone()))
                .map_or_else(
                    || (id.clone(), None),
                    |entry| (entry.model.name.clone(), Some(entry.model.reasoning)),
                );
            out.push(PiModelListing {
                id: full_id,
                name,
                thinking,
            });
        }
    }
}

fn append_static_models_without_live(
    out: &mut Vec<PiModelListing>,
    seen: &mut HashSet<String>,
    registry: &ModelRegistry,
    live_by_provider: &HashMap<String, Vec<String>>,
) {
    for entry in registry.models() {
        let provider = entry.model.provider.as_str();
        if live_by_provider.contains_key(provider) {
            continue;
        }
        let full_id = format!("{provider}/{}", entry.model.id);
        if seen.insert(full_id.clone()) {
            out.push(PiModelListing {
                id: full_id,
                name: entry.model.name.clone(),
                thinking: Some(entry.model.reasoning),
            });
        }
    }
}
