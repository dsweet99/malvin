
use super::models_cmd::{run_models, ModelsArgs};

#[cfg(unix)]
fn write_exec(path: &std::path::Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, body).expect("write");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
fn restore_malvin_pi(prior: Option<std::ffi::OsString>) {
    #[allow(unsafe_code)]
    unsafe {
        match prior {
            Some(v) => std::env::set_var("MALVIN_PI", v),
            None => std::env::remove_var("MALVIN_PI"),
        }
    }
}

#[cfg(unix)]
fn assert_live_auth_filter(out: &str) {
    assert!(
        !out.contains("pi:openai/gpt-4o"),
        "openai should be filtered without OPENAI_API_KEY: {out}"
    );
    assert!(
        out.contains("pi:openrouter/claude"),
        "openrouter should remain when OPENROUTER_API_KEY is set: {out}"
    );
    assert!(
        out.contains("pi:llamacpp/local"),
        "provider with no auth env keys should remain: {out}"
    );
}

const FAKE_PI_LIST_SCRIPT: &str = "#!/bin/sh
if [ \"$1\" = --list-providers ]; then
cat <<'EOF'
provider                name                       aliases                                                                auth env                                                                                                              api                    
----------------------  -------------------------  ---------------------------------------------------------------------  --------------------------------------------------------------------------------------------------------------------  -----------------------
openai                  OpenAI                                                                                            OPENAI_API_KEY                                                                                                        openai-responses       
openrouter              OpenRouter                 open-router                                                            OPENROUTER_API_KEY                                                                                                    openai-completions     
llamacpp                llama.cpp                  llama-cpp                                                                                                                                                           openai-completions     
EOF
exit 0
fi
cat <<'EOF'
provider        model                                                     context
openai          gpt-4o                                                    128K
openrouter      claude                                                    200K
llamacpp        local                                                     8K
EOF
";

#[cfg(unix)]
fn install_fake_pi_listing(dir: &std::path::Path) -> std::path::PathBuf {
    write_exec(&dir.join("agent"), "#!/bin/sh\nexit 1\n");
    let pi = dir.join("pi");
    write_exec(&pi, FAKE_PI_LIST_SCRIPT);
    pi
}

#[cfg(unix)]
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

#[cfg(unix)]
#[test]
fn run_models_filters_pi_rows_using_live_provider_auth_map() {
    use crate::output::{enable_stdout_capture, take_captured_stdout};
    use crate::repo_checks::set_fake_command_dir;
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    let pi = install_fake_pi_listing(tmp.path());
    let _guard = set_fake_command_dir(tmp.path());
    let prior_pi = std::env::var_os("MALVIN_PI");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI", pi.as_os_str());
    }
    enable_stdout_capture();
    run_models_pi_only_with_openrouter_key();
    let out = take_captured_stdout();
    restore_malvin_pi(prior_pi);
    assert_live_auth_filter(&out);
}

const FAKE_PI_PROVIDERS_FAIL_SCRIPT: &str = "#!/bin/sh
if [ \"$1\" = --list-providers ]; then
echo fail >&2
exit 1
fi
cat <<'EOF'
provider        model                                                     context
openai          gpt-4o                                                    128K
EOF
";

#[cfg(unix)]
#[test]
fn run_models_prints_pi_rows_when_provider_map_unavailable() {
    use crate::output::{enable_stdout_capture, take_captured_stdout};
    use crate::repo_checks::set_fake_command_dir;
    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    write_exec(&tmp.path().join("agent"), "#!/bin/sh\nexit 1\n");
    let pi = tmp.path().join("pi");
    write_exec(&pi, FAKE_PI_PROVIDERS_FAIL_SCRIPT);
    let _guard = set_fake_command_dir(tmp.path());
    let prior_pi = std::env::var_os("MALVIN_PI");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI", pi.as_os_str());
    }
    enable_stdout_capture();
    run_models(
        ModelsArgs {
            words: vec!["pi:".into()],
        },
        crate::config::DEFAULT_CLI_MODEL,
    )
    .expect("unfiltered models");
    let out = take_captured_stdout();
    restore_malvin_pi(prior_pi);
    assert!(
        out.contains("pi provider auth map unavailable"),
        "should note missing map: {out}"
    );
    assert!(
        out.contains("pi:openai/gpt-4o"),
        "should still list pi models: {out}"
    );
}
