use super::{
    DEFAULT_PI_LIST_MODELS_TIMEOUT_MS, list_pi_models_sync, pi_list_models_timeout,
};

#[test]
fn pi_list_models_timeout_env_clamps_and_defaults() {
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let prior = std::env::var_os("MALVIN_PI_LIST_MODELS_TIMEOUT_MS");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "0");
    }
    assert_eq!(
        pi_list_models_timeout(),
        std::time::Duration::from_millis(1),
        "zero must clamp to at least 1ms"
    );
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "not-a-number");
    }
    assert_eq!(
        pi_list_models_timeout(),
        std::time::Duration::from_millis(DEFAULT_PI_LIST_MODELS_TIMEOUT_MS)
    );
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", "250");
    }
    assert_eq!(
        pi_list_models_timeout(),
        std::time::Duration::from_millis(250)
    );
    #[allow(unsafe_code)]
    unsafe {
        match prior {
            Some(v) => std::env::set_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS", v),
            None => std::env::remove_var("MALVIN_PI_LIST_MODELS_TIMEOUT_MS"),
        }
    }
}

#[test]
fn list_pi_models_sync_reads_crate_registry() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    crate::test_utils::with_isolated_home(|_| {
        let _saved = crate::test_utils::SavedEnvVars::capture(&[
            "PI_CODING_AGENT_DIR",
            "OPENAI_API_KEY",
            "OPENROUTER_API_KEY",
            "ANTHROPIC_API_KEY",
        ]);
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("PI_CODING_AGENT_DIR", tmp.path());
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("OPENROUTER_API_KEY");
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
        let rows = list_pi_models_sync(false).expect("crate registry");
        assert!(
            rows.iter().any(|row| row.id.contains('/')),
            "expected provider/model ids, got {rows:?}"
        );
    });
}
