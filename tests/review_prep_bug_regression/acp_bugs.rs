use super::helpers::manifest_root;

fn read_repo_file(rel: &str) -> String {
    std::fs::read_to_string(manifest_root().join(rel)).unwrap_or_else(|e| {
        panic!("read {rel}: {e}");
    })
}

fn kissignore_lists(rel: &str) -> bool {
    read_repo_file(".kissignore")
        .lines()
        .map(str::trim)
        .any(|line| line == rel)
}

/// [4] `src/acp_session_tests/` dropped `unix_helpers.rs`, `unix_shutdown.rs`, `linux_spawn_abort.rs`.
#[test]
fn acp_session_tests_must_include_unix_integration_specs() {
    let root = manifest_root().join("src/acp_session_tests");
    for rel in [
        "unix_helpers.rs",
        "unix_shutdown.rs",
        "linux_spawn_abort.rs",
    ] {
        let path = root.join(rel);
        assert!(
            path.is_file(),
            "bug: missing {rel} under src/acp_session_tests/ (present on HEAD; Unix \
             cgroup/shutdown coverage removed)"
        );
    }
}

/// [3] `acp_session_cancel_clears_busy_state_after_rpc_error` duplicated in two files.
#[test]
fn session_cancel_test_must_exist_in_only_one_compiled_location() {
    let session_tests = read_repo_file("src/acp/session_tests.rs");
    let cancel_rs = read_repo_file("src/acp_session_tests/cancel.rs");
    let needle = "async fn acp_session_cancel_clears_busy_state_after_rpc_error";
    let in_session_tests = session_tests.contains(needle);
    let in_cancel_rs = cancel_rs.contains(needle);
    assert!(
        in_session_tests ^ in_cancel_rs,
        "bug: cancel test must live in exactly one of session_tests.rs (kissignored) or \
         acp_session_tests/cancel.rs (compiled), not both (in_session_tests={in_session_tests}, \
         in_cancel_rs={in_cancel_rs})"
    );
}

/// [3] `#[path = "session_tests.rs"]` without `.kissignore` entry yields kiss `orphan_module`.
#[test]
fn session_tests_path_module_requires_kissignore_entry() {
    let session_rs = read_repo_file("src/acp/session.rs");
    let uses_path_module = session_rs.contains("#[path = \"session_tests.rs\"]");
    if !uses_path_module {
        return;
    }
    assert!(
        kissignore_lists("src/acp/session_tests.rs"),
        "bug: session.rs uses #[path = \"session_tests.rs\"] but .kissignore does not list \
         src/acp/session_tests.rs — kiss reports orphan_module when only target/ is ignored"
    );
}
