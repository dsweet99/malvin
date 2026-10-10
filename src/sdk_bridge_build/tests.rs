#![cfg_attr(test, allow(unsafe_code))]

use super::*;

fn run_build_script_honors_lld_opt_out() {
    let _g = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("MALVIN_DISABLE_LLD", Some("1"), run_build_script);
    crate::agent_process::with_env(
        "MALVIN_DISABLE_LLD",
        Some("1"),
        lld::emit_fast_bin_linker_args,
    );
    if let Some(dir) = lld::rustc_gcc_ld_dir() {
        assert!(
            dir.components().any(|c| c.as_os_str() == "gcc-ld"),
            "unexpected gcc-ld path: {}",
            dir.display()
        );
    }
}

fn run_build_script_runs_on_docs_rs() {
    let _g = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("DOCS_RS", Some("1"), run_build_script);
}

#[test]
fn kiss_bundled_sdk_bridge_build_tests() {
    run_build_script_honors_lld_opt_out();
    run_build_script_runs_on_docs_rs();
}
