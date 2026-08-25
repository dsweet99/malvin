use std::path::{Path, PathBuf};

use crate::acp::AuthError;

use super::discover::resolve_codex_bin;

pub fn ensure_codex_authenticated() -> Result<(), AuthError> {
    resolve_codex_bin().map_err(AuthError)?;
    if has_codex_login() {
        return Ok(());
    }
    Err(AuthError(
        "Codex backend is not authenticated. Run `codex login` or set OPENAI_API_KEY. Expected login state in $CODEX_HOME/auth.json (default ~/.codex/auth.json).".into(),
    ))
}

fn has_codex_login() -> bool {
    crate::acp::env_key_nonempty("OPENAI_API_KEY") || auth_file_has_login(&codex_auth_path())
}

fn codex_auth_path() -> PathBuf {
    if let Some(home) = std::env::var_os("CODEX_HOME").filter(|s| !s.is_empty()) {
        return PathBuf::from(home).join("auth.json");
    }
    crate::user_home::user_home_dir()
        .join(".codex")
        .join("auth.json")
}

fn auth_file_has_login(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    nonempty_json_str(value.get("OPENAI_API_KEY"))
        || nonempty_json_str(value.pointer("/tokens/access_token"))
        || nonempty_json_str(value.pointer("/tokens/refresh_token"))
}

fn nonempty_json_str(value: Option<&serde_json::Value>) -> bool {
    value
        .and_then(serde_json::Value::as_str)
        .is_some_and(|s| !s.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_file_accepts_tokens_or_key() {
        let tmp = tempfile::tempdir().expect("tmp");
        let path = tmp.path().join("auth.json");
        assert!(!auth_file_has_login(&path));
        std::fs::write(&path, r#"{"tokens":{"access_token":"tok"}}"#).expect("write");
        assert!(auth_file_has_login(&path));
        std::fs::write(&path, r#"{"OPENAI_API_KEY":"sk"}"#).expect("write");
        assert!(auth_file_has_login(&path));
        std::fs::write(&path, r#"{"tokens":{}}"#).expect("write");
        assert!(!auth_file_has_login(&path));
    }

    #[test]
    fn ensure_codex_authenticated_ok_with_env_key() {
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().expect("tmp");
        crate::acp::with_env(
            "CODEX_HOME",
            Some(tmp.path().to_str().expect("utf8")),
            || {
                crate::acp::with_env("OPENAI_API_KEY", Some("test-key"), || {
                    crate::acp::with_env("MALVIN_CODEX", None, || {
                        if resolve_codex_bin().is_ok() {
                            assert!(ensure_codex_authenticated().is_ok());
                        }
                    });
                });
            },
        );
    }
}
