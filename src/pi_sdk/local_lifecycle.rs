pub(crate) fn ensure_local_llm(provider: &str, model: &str) -> Result<(), String> {
    super::local_llm_client::ensure_via_manager(provider, model)
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
pub fn model_needs_local_llm(model: &crate::model_id::ParsedModel) -> bool {
    model
        .pi_provider_and_model()
        .is_some_and(|(provider, _)| pi::provider_metadata::provider_is_keyless_local(provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model_id::parse_model_id;

    #[test]
    fn model_needs_local_llm_detects_ollama_and_skips_cursor() {
        let local = parse_model_id("rpi:ollama/tiny").expect("parse");
        let cursor = parse_model_id("cursor:auto").expect("parse");
        assert!(model_needs_local_llm(&local));
        assert!(!model_needs_local_llm(&cursor));
    }

    #[test]
    fn local_alias_needs_local_llm() {
        let local = parse_model_id("rpi:local/malvin-qwen14:latest").expect("parse");
        assert!(model_needs_local_llm(&local));
    }
}
