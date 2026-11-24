use super::discover::ModelListPage;
use super::discover::{CODEX_MISSING_HINT, resolve_codex_bin};

#[test]
fn discover_names_and_malvin_codex_override_must_be_executable() {
    let page = ModelListPage::empty();
    assert!(page.models.is_empty());
    let _ = ModelListPage::empty();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _lock = crate::test_support::test_utils::test_env_lock();
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("codex");
        std::fs::write(&path, "").expect("write");
        let set_mode = |mode| {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).expect("chmod");
        };
        let resolve = |value: &std::path::Path| {
            let mut out = Err(String::new());
            crate::agent_process::with_env("MALVIN_CODEX", value.to_str(), || {
                out = resolve_codex_bin();
            });
            out
        };
        set_mode(0o755);
        assert_eq!(resolve(&path), Ok(path.clone()));
        set_mode(0o644);
        let err = resolve(&path).unwrap_err();
        assert!(
            err.contains("MALVIN_CODEX is set but not executable")
                && err.contains(CODEX_MISSING_HINT)
        );
        let err = resolve(&dir.path().join("missing")).unwrap_err();
        assert!(err.contains("MALVIN_CODEX is set but not a file"), "{err}");
    }
}
