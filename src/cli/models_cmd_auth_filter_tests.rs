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
        out.contains("pi:zhipuai/"),
        "provider absent from the env-key map should remain: {out}"
    );
    // Keyless local providers (llamacpp et al.) have no registry listing rows
    // on crates.io `pi_agent_rust` 0.1.23: `ad_hoc_model_entry` synthesizes
    // entries only on the per-request resolve path, never in
    // `load_for_listing`. Their access rule therefore lives at the auth layer,
    // not in the printed table.
    assert!(
        crate::pi_sdk::is_provider_authenticated("llamacpp"),
        "keyless local provider should count as authenticated without env keys"
    );
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
