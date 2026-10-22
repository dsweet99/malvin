use super::ensure_claude_authenticated;

#[test]
fn claude_auth_accepts_anthropic_api_key() {
    let _lock = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("ANTHROPIC_API_KEY", Some("test-key"), || {
        assert!(ensure_claude_authenticated().is_ok());
    });
}

#[test]
fn claude_auth_reports_missing_credentials() {
    let _lock = crate::test_support::test_utils::test_env_lock();
    let home = tempfile::tempdir().expect("temporary home");
    let home = home.path().to_str().expect("UTF-8 temporary home");
    crate::agent_process::with_env("HOME", Some(home), || {
        crate::agent_process::with_env("ANTHROPIC_API_KEY", None, || {
            crate::agent_process::with_env("ANTHROPIC_OAUTH_TOKEN", None, || {
                crate::agent_process::with_env("ANTHROPIC_AUTH_TOKEN", None, || {
                    crate::agent_process::with_env("CLAUDE_CODE_OAUTH_TOKEN", None, || {
                        let error = ensure_claude_authenticated().expect_err("missing auth");
                        assert!(error.0.contains("ANTHROPIC_API_KEY"));
                    });
                });
            });
        });
    });
}
