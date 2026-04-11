fn clear_cursor_cred_env() {
    unsafe {
        std::env::remove_var("CURSOR_API_KEY");
        std::env::remove_var("CURSOR_AUTH_TOKEN");
    }
}

#[test]
fn api_key_explicit_nonempty() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    assert_eq!(
        effective_cursor_api_key(Some("k")).as_deref(),
        Some("k")
    );
}

#[test]
fn api_key_trims_whitespace_and_bom_on_explicit() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    assert_eq!(
        effective_cursor_api_key(Some("  abc  ")).as_deref(),
        Some("abc")
    );
    assert_eq!(
        effective_cursor_api_key(Some("\u{feff}key_123")).as_deref(),
        Some("key_123")
    );
}

#[test]
fn api_key_empty_explicit_falls_back_to_env() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "from-env-after-empty-explicit");
    }
    assert_eq!(
        effective_cursor_api_key(Some("")).as_deref(),
        Some("from-env-after-empty-explicit")
    );
}

#[test]
fn api_key_explicit_none_reads_env() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "from-env");
    }
    assert_eq!(
        effective_cursor_api_key(None).as_deref(),
        Some("from-env")
    );
}

#[test]
fn api_key_explicit_nonempty_beats_env() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "e");
    }
    assert_eq!(
        effective_cursor_api_key(Some("explicit")).as_deref(),
        Some("explicit")
    );
}

#[test]
fn api_key_env_only() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "env");
    }
    assert_eq!(effective_cursor_api_key(None).as_deref(), Some("env"));
}

#[test]
fn api_key_empty_env_ignored_when_explicit() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_API_KEY", "");
    }
    assert_eq!(
        effective_cursor_api_key(Some("x")).as_deref(),
        Some("x")
    );
}

#[test]
fn auth_token_explicit_nonempty() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    assert_eq!(
        effective_cursor_auth_token(Some("t")).as_deref(),
        Some("t")
    );
}

#[test]
fn auth_token_explicit_none_reads_env() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "tok-env");
    }
    assert_eq!(
        effective_cursor_auth_token(None).as_deref(),
        Some("tok-env")
    );
}

#[test]
fn auth_token_explicit_nonempty_beats_env() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "x");
    }
    assert_eq!(
        effective_cursor_auth_token(Some("explicit")).as_deref(),
        Some("explicit")
    );
}

#[test]
fn auth_token_env_only() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "env");
    }
    assert_eq!(
        effective_cursor_auth_token(None).as_deref(),
        Some("env")
    );
}

#[test]
fn auth_token_empty_env_ignored_when_explicit() {
    let _g = test_env_lock();
    clear_cursor_cred_env();
    unsafe {
        std::env::set_var("CURSOR_AUTH_TOKEN", "");
    }
    assert_eq!(
        effective_cursor_auth_token(Some("y")).as_deref(),
        Some("y")
    );
}

const ARBITRARY_ENV: &str = "MALVIN_TEST_ARBITRARY_CURSOR_ENV_1";

#[test]
fn nonempty_explicit_or_env_var_none_and_missing_env() {
    let _g = test_env_lock();
    unsafe {
        std::env::remove_var(ARBITRARY_ENV);
    }
    assert!(nonempty_explicit_or_env_var(None, ARBITRARY_ENV).is_none());
}

#[test]
fn nonempty_explicit_or_env_var_env_only() {
    let _g = test_env_lock();
    unsafe {
        std::env::remove_var(ARBITRARY_ENV);
        std::env::set_var(ARBITRARY_ENV, "arbitrary");
    }
    assert_eq!(
        nonempty_explicit_or_env_var(None, ARBITRARY_ENV).as_deref(),
        Some("arbitrary")
    );
    unsafe {
        std::env::remove_var(ARBITRARY_ENV);
    }
}

#[test]
fn nonempty_explicit_or_env_var_empty_env() {
    let _g = test_env_lock();
    unsafe {
        std::env::set_var(ARBITRARY_ENV, "");
    }
    assert!(nonempty_explicit_or_env_var(None, ARBITRARY_ENV).is_none());
    unsafe {
        std::env::remove_var(ARBITRARY_ENV);
    }
}

#[test]
fn nonempty_explicit_or_env_var_explicit_wins() {
    let _g = test_env_lock();
    unsafe {
        std::env::set_var(ARBITRARY_ENV, "env");
    }
    assert_eq!(
        nonempty_explicit_or_env_var(Some("ex"), ARBITRARY_ENV).as_deref(),
        Some("ex")
    );
    unsafe {
        std::env::remove_var(ARBITRARY_ENV);
    }
}

#[test]
fn nonempty_explicit_or_env_var_empty_explicit_falls_back() {
    let _g = test_env_lock();
    unsafe {
        std::env::set_var(ARBITRARY_ENV, "fallback");
    }
    assert_eq!(
        nonempty_explicit_or_env_var(Some(""), ARBITRARY_ENV).as_deref(),
        Some("fallback")
    );
    unsafe {
        std::env::remove_var(ARBITRARY_ENV);
    }
}
