use std::path::PathBuf;

#[path = "catalog.rs"]
pub(crate) mod catalog;
#[path = "model_list.rs"]
mod model_list;

use catalog::{CatalogChild, list_models_from_child, spawn_codex_model_server};

#[derive(Debug)]
pub(crate) struct ModelListPage {
    pub(crate) models: Vec<(String, String)>,
    pub(crate) next_cursor: Option<String>,
}

pub const CODEX_MISSING_HINT: &str = "codex backend requires the codex binary on PATH (or MALVIN_CODEX); install Codex CLI separately.";
pub const DEFAULT_CODEX_LIST_MODELS_TIMEOUT_MS: u64 = 30_000;

#[must_use]
pub fn codex_missing_binary_message() -> String {
    CODEX_MISSING_HINT.to_string()
}

pub fn resolve_codex_bin() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MALVIN_CODEX") {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err(format!(
                "MALVIN_CODEX points to a missing file ({}); {CODEX_MISSING_HINT}",
                path.display()
            ));
        }
        if !codex_path_is_executable(&path) {
            return Err(format!(
                "MALVIN_CODEX is not executable ({}); {CODEX_MISSING_HINT}",
                path.display()
            ));
        }
        return Ok(path);
    }
    crate::support_paths::lookup_bin_on_path("codex").ok_or_else(codex_missing_binary_message)
}

#[must_use]
pub(crate) fn codex_path_is_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

pub fn list_codex_models() -> Result<Vec<(String, String)>, String> {
    let mut catalog = CatalogChild::wrap(spawn_codex_model_server()?);
    list_models_from_child(&mut catalog.child)
}

#[cfg(test)]
pub(crate) fn models_from_list_response(
    value: &serde_json::Value,
) -> Result<Vec<(String, String)>, String> {
    model_list::parse_model_list_page(value).map(|page| page.models)
}

pub(crate) fn model_list_params(cursor: Option<&str>) -> serde_json::Value {
    serde_json::json!({"limit": 100, "includeHidden": true, "cursor": cursor})
}

#[cfg(test)]
impl ModelListPage {
    pub(crate) const fn empty() -> Self {
        Self {
            models: Vec::new(),
            next_cursor: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_list_includes_hidden() {
        let params = model_list_params(None);
        assert_eq!(params["includeHidden"], true);
        assert_eq!(params["limit"], 100);
    }

    #[test]
    fn models_from_list_response_reads_ids() {
        let value = serde_json::json!({
            "result": {"data": [{"id": "gpt-reserve", "displayName": "Reserve"}]}
        });
        assert_eq!(
            models_from_list_response(&value).unwrap(),
            vec![("gpt-reserve".into(), "Reserve".into())]
        );
    }

    #[test]
    fn test_codex_missing_binary_message() {
        assert!(codex_missing_binary_message().contains("MALVIN_CODEX"));
    }

    #[test]
    fn kiss_cov_discover() {
        let page = ModelListPage::empty();
        assert!(page.models.is_empty());
        let _ = model_list_params(None);
        let _ = models_from_list_response(&serde_json::json!({"result":{"data":[]}}));
        let _ = list_codex_models();
        let _ = resolve_codex_bin();
        let _ = codex_missing_binary_message();
    }

    #[test]
    fn model_list_page_fields_are_readable() {
        let page = ModelListPage {
            models: vec![("gpt-reserve".into(), "Reserve".into())],
            next_cursor: Some("n".into()),
        };
        assert_eq!(page.models[0].0, "gpt-reserve");
        assert_eq!(page.next_cursor.as_deref(), Some("n"));
    }

    #[cfg(unix)]
    #[test]
    fn test_list_codex_models() {
        use std::os::unix::fs::PermissionsExt;
        let _lock = crate::test_utils::test_env_lock();
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "#!/bin/sh\nprintf '%s\\n' '{\"id\":1,\"result\":{}}' '{\"id\":2,\"result\":{\"data\":[{\"id\":\"gpt-test\",\"displayName\":\"Test\"}]}}'\n").unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        crate::acp::with_env("MALVIN_CODEX", Some(p.to_str().unwrap()), || {
            assert_eq!(
                list_codex_models().unwrap(),
                vec![("gpt-test".into(), "Test".into())]
            );
            assert_eq!(
                list_codex_models().unwrap()[0].0,
                "gpt-test"
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn hung_codex_model_list_times_out() {
        use std::os::unix::fs::PermissionsExt;
        use std::time::{Duration, Instant};
        let _lock = crate::test_utils::test_env_lock();
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "#!/bin/sh\nsleep 30\n").unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        crate::acp::with_env("MALVIN_CODEX", Some(p.to_str().unwrap()), || {
            crate::acp::with_env("MALVIN_CODEX_LIST_MODELS_TIMEOUT_MS", Some("200"), || {
                let started = Instant::now();
                let err = list_codex_models().expect_err("must time out");
                assert!(err.contains("timed out"), "got: {err}");
                assert!(started.elapsed() < Duration::from_secs(2));
            });
        });
    }
}
