use std::collections::HashSet;

use super::provider_metadata::provider_is_keyless_local;

fn display_id(id: &str) -> String {
    let provider = id.split('/').next().unwrap_or("");
    if provider_is_keyless_local(provider) {
        format!("local/{id}")
    } else {
        id.to_string()
    }
}

#[must_use]
pub fn filter_local_listings(models: Vec<(String, String)>) -> Vec<(String, String)> {
    merge_listings(models, local_display_models())
}

fn merge_listings(
    models: Vec<(String, String)>,
    discovered: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let mut seen = HashSet::new();
    super::local_llms_config::filter_listings_by_local_llms_config(models)
        .into_iter()
        .chain(discovered)
        .map(|(id, detail)| (display_id(&id), detail))
        .filter(|(id, _)| seen.insert(id.clone()))
        .collect()
}

#[must_use]
pub(crate) fn local_display_models() -> Vec<(String, String)> {
    let tags = super::local_llm_ollama::list_ollama_models()
        .into_iter()
        .map(|name| (format!("ollama/{name}"), format!("{name}\tlocal")))
        .collect();
    super::local_llms_config::filter_listings_by_local_llms_config(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_ids_get_local_prefix_and_duplicates_drop() {
        crate::test_utils::with_isolated_home(|_| {
            let out = merge_listings(
                vec![
                    ("openai/gpt-4o".into(), "gpt-4o\tthinking=no".into()),
                    ("ollama/qwen:1b".into(), "qwen:1b\tthinking=no".into()),
                ],
                vec![("ollama/qwen:1b".into(), "qwen:1b\tlocal".into())],
            );
            assert_eq!(out.len(), 2);
            assert_eq!(out[0].0, "openai/gpt-4o");
            assert_eq!(out[1].0, "local/ollama/qwen:1b");
            assert!(out[1].1.contains("thinking=no"));
        });
    }
}
