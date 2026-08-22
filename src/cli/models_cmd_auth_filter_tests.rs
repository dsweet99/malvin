use super::models_cmd::{ModelsArgs, run_models};

fn isolated_pi_home(tmp: &std::path::Path) -> String {
    tmp.join("pi-home").to_string_lossy().into_owned()
}

fn run_models_pi_only_with_openrouter_key() {
    crate::acp::with_env("OPENAI_API_KEY", None, || {
        crate::acp::with_env("OPENROUTER_API_KEY", Some("k"), || {
            run_models(
                ModelsArgs {
                    words: vec!["pi:".into()],
                },
                crate::config::DEFAULT_CLI_MODEL,
            )
            .expect("filtered models");
        });
    });
}

fn assert_live_auth_filter(out: &str) {
    assert!(
        !out.contains("pi:openai/"),
        "openai should be filtered without OPENAI_API_KEY: {out}"
    );
    assert!(
        out.contains("pi:openrouter/"),
        "openrouter should remain when OPENROUTER_API_KEY is set: {out}"
    );
    assert!(
        !out.contains("pi:zhipuai/"),
        "provider whose API key is unset must be hidden: {out}"
    );
    assert!(
        !out.contains("pi:cohere/"),
        "provider whose API key is unset must be hidden: {out}"
    );
}

#[test]
fn run_models_filters_pi_rows_using_stored_credentials_too() {
    use crate::output::{enable_stdout_capture, take_captured_stdout};
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = isolated_pi_home(tmp.path());
    crate::acp::with_env("PI_CODING_AGENT_DIR", Some(&home), || {
        // Store an openai credential in pi's auth file only (no env keys).
        {
            let auth_path = pi::sdk::Config::auth_path();
            let mut auth =
                pi::auth::AuthStorage::load(auth_path.clone()).expect("load auth storage");
            auth.set(
                "openai",
                pi::auth::AuthCredential::ApiKey {
                    key: "stored-key".to_string(),
                },
            );
            auth.save().expect("save auth storage");
        }
        enable_stdout_capture();
        run_models_pi_only_with_openrouter_key();
        let out = take_captured_stdout();
        assert!(
            out.contains("pi:openai/"),
            "stored credential must list the same providers a run would accept: {out}"
        );
        assert!(
            out.contains("pi:openrouter/"),
            "env-key provider must still list: {out}"
        );
    });
}

#[test]
fn run_models_filters_pi_rows_using_live_provider_auth_map() {
    use crate::output::{enable_stdout_capture, take_captured_stdout};
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = isolated_pi_home(tmp.path());
    crate::acp::with_env("PI_CODING_AGENT_DIR", Some(&home), || {
        enable_stdout_capture();
        run_models_pi_only_with_openrouter_key();
        let out = take_captured_stdout();
        assert_live_auth_filter(&out);
    });
}

#[test]
fn run_models_lists_pi_rows_without_pi_binary() {
    use crate::output::{enable_stdout_capture, take_captured_stdout};
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let home = isolated_pi_home(tmp.path());
    crate::acp::with_env("PI_CODING_AGENT_DIR", Some(&home), || {
        crate::acp::with_env("OPENROUTER_API_KEY", Some("k"), || {
            enable_stdout_capture();
            run_models(
                ModelsArgs {
                    words: vec!["pi:".into()],
                },
                crate::config::DEFAULT_CLI_MODEL,
            )
            .expect("crate models");
            let out = take_captured_stdout();
            assert!(
                !out.contains("pi models unavailable"),
                "crate registry should list models: {out}"
            );
            assert!(
                out.contains("pi:openrouter/"),
                "should list pi models: {out}"
            );
        });
    });
}
