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
    fn model_needs_local_llm_detects_ollama_and_skips_cursor_and_prepare_local_llm_is_a_noop_for_cloud_providers()
     {
        {
            let local = parse_model_id("pi:ollama/tiny").expect("parse");
            let cursor = parse_model_id("cursor:auto").expect("parse");
            assert!(model_needs_local_llm(&local));
            assert!(!model_needs_local_llm(&cursor));
        }
        {
            crate::test_support::test_utils::with_isolated_home(|home| {
                prepare_local_llm(home, "openai", "gpt-4o").expect("noop");
                assert!(!super::super::provider_metadata::pi_models_json_path().exists());
            });
        }
    }
    #[test]
    fn local_output_cap_only_applies_to_keyless_local_providers_and_local_alias_needs_local_llm() {
        {
            crate::test_support::test_utils::with_isolated_home(|home| {
                assert!(local_output_cap(home, "openai", "gpt-4o").is_none());
                let cap = local_output_cap(home, "ollama", "tiny").expect("local cap");
                assert!(cap > 0);
            });
        }
        {
            let local = parse_model_id("pi:local/malvin-qwen14:latest").expect("parse");
            assert!(model_needs_local_llm(&local));
        }
    }

    #[test]
    fn housekeep_removes_stale_manager_files_and_keeps_a_live_lock() {
        let _guard = crate::local_llm::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tmp");
        housekeep_case(&tmp.path().join("stale"), "999999999", false);
        housekeep_case(
            &tmp.path().join("live"),
            &std::process::id().to_string(),
            true,
        );
        assert!(super::super::local_llm_ollama::process_alive(std::process::id()));
    }

    fn housekeep_case(dir: &std::path::Path, pid: &str, keep: bool) {
        std::fs::create_dir_all(dir).expect("dir");
        let sock = dir.join("manager.sock");
        let lock = dir.join("manager.lock");
        std::fs::write(&sock, b"sock").expect("sock");
        std::fs::write(&lock, format!("{pid}\n")).expect("lock");
        let sock_s = sock.to_str().expect("utf8").to_string();
        let lock_s = lock.to_str().expect("utf8").to_string();
        crate::agent_process::with_env("MALVIN_LOCAL_LLM_MANAGER_SOCK", Some(&sock_s), || {
            crate::agent_process::with_env("MALVIN_LOCAL_LLM_MANAGER_LOCK", Some(&lock_s), || {
                housekeep_local_llms();
            });
        });
        assert_eq!(sock.exists(), keep, "socket keep={keep} at {}", sock.display());
        assert_eq!(lock.exists(), keep, "lock keep={keep} at {}", lock.display());
    }

    #[test]
    fn ensure_local_llm_returns_the_managers_down_provider_error() {
        let Some(provider) = super::super::local_llm_ollama::down_non_ollama_provider() else {
            return;
        };
        let _guard = crate::local_llm::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tmp");
        let sock = tmp.path().join("ensure.sock");
        let lock = tmp.path().join("ensure.lock");
        let sock_s = sock.to_str().expect("utf8").to_string();
        let lock_s = lock.to_str().expect("utf8").to_string();
        crate::agent_process::with_env("MALVIN_LOCAL_LLM_MANAGER_SOCK", Some(&sock_s), || {
            crate::agent_process::with_env("MALVIN_LOCAL_LLM_MANAGER_LOCK", Some(&lock_s), || {
                crate::agent_process::with_env("MALVIN_TIME_SINCE_LAST_CALL_SECONDS", Some("1"), || {
                    let handle =
                        std::thread::spawn(super::super::local_llm_daemon::run_local_llm_manager);
                    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
                    while std::time::Instant::now() < deadline && !sock.exists() {
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    assert!(sock.exists(), "manager should create {}", sock.display());
                    let err = ensure_local_llm(provider, "x").expect_err("down provider");
                    assert!(
                        err.contains("auto-start only `ollama`"),
                        "ensure should report that only ollama can be started: {err}"
                    );
                    handle.join().expect("join").expect("manager ok");
                });
            });
        });
    }
}
