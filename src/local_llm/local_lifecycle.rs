use std::path::Path;

pub(crate) fn ensure_local_llm(provider: &str, model: &str) -> Result<(), String> {
    super::local_llm_client::ensure_via_manager(provider, model)
}

fn local_context_size(cwd: &Path, provider: &str, model: &str) -> u32 {
    super::local_llms_config::context_size_override(provider, model)
        .unwrap_or_else(|| super::local_context::context_size_for_workdir(cwd))
}

#[must_use]
pub(crate) fn local_output_cap(cwd: &Path, provider: &str, model: &str) -> Option<u64> {
    super::provider_metadata::provider_is_keyless_local(provider).then(|| {
        u64::from(super::local_context::max_tokens_for_context(
            local_context_size(cwd, provider, model),
        ))
    })
}

pub(crate) fn prepare_local_llm(cwd: &Path, provider: &str, model: &str) -> Result<(), String> {
    if !super::provider_metadata::provider_is_keyless_local(provider) {
        return Ok(());
    }
    let context_size = local_context_size(cwd, provider, model);
    super::local_context::ensure_capped_local_model_catalog(provider, model, context_size)?;
    ensure_local_llm(provider, model)
}

pub(crate) fn hold_local_llm() -> Result<(), String> {
    super::local_llm_client::hold_via_manager()
}

pub(crate) fn release_local_llm() -> Result<(), String> {
    super::local_llm_client::release_via_manager()
}

pub fn housekeep_local_llms() {
    super::local_llm_daemon::reclaim_stale_manager_files();
}

#[must_use]
pub fn model_needs_local_llm(model: &crate::config::model_id::ParsedModel) -> bool {
    model
        .pi_provider_and_model()
        .is_some_and(|(provider, _)| super::provider_metadata::provider_is_keyless_local(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model_id::parse_model_id;

    #[test]
    fn model_needs_local_llm_detects_ollama_and_skips_cursor() {
        let local = parse_model_id("pi:ollama/tiny").expect("parse");
        let cursor = parse_model_id("cursor:auto").expect("parse");
        assert!(model_needs_local_llm(&local));
        assert!(!model_needs_local_llm(&cursor));
    }

    #[test]
    fn prepare_local_llm_is_a_noop_for_cloud_providers() {
        crate::test_support::test_utils::with_isolated_home(|home| {
            prepare_local_llm(home, "openai", "gpt-4o").expect("noop");
            assert!(!super::super::provider_metadata::pi_models_json_path().exists());
        });
    }

    #[test]
    fn local_output_cap_only_applies_to_keyless_local_providers() {
        crate::test_support::test_utils::with_isolated_home(|home| {
            assert!(local_output_cap(home, "openai", "gpt-4o").is_none());
            let cap = local_output_cap(home, "ollama", "tiny").expect("local cap");
            assert!(cap > 0);
        });
    }

    #[test]
    fn local_alias_needs_local_llm() {
        let local = parse_model_id("pi:local/malvin-qwen14:latest").expect("parse");
        assert!(model_needs_local_llm(&local));
    }
}
