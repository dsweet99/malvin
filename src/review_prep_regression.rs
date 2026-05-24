const INIT_COMMIT_NOTICE: &str =
    "init: creating initial commit (skipping pre-commit hooks to avoid bootstrap cycle)";
const SESSION_SPAWN_INC: &str = include_str!("acp/session_spawn.inc");

#[test]
fn captured_stderr_must_use_thread_local_buffer() {
    let output_src = include_str!("output/mod.rs");
    assert!(
        output_src.contains("thread_local!"),
        "stderr capture must use thread-local storage to avoid parallel test races"
    );
}

#[test]
fn init_tracing_fallback_must_install_globally_not_thread_local_only() {
    let src = include_str!("tracing_init.rs");
    assert!(
        !src.contains("dispatcher::set_default"),
        "when try_init fails, MalvinLogLayer must be installed for all threads, not via thread-local set_default"
    );
}

#[test]
fn init_tracing_must_not_discard_try_init_errors() {
    let src = include_str!("tracing_init.rs");
    assert!(
        !src.contains("let _ = tracing_subscriber::registry()"),
        "init_tracing must handle try_init() failure instead of silently discarding it"
    );
}

#[test]
fn malvin_must_install_tracing_subscriber_so_warn_events_are_visible() {
    crate::init_from_env();
    assert!(
        tracing::dispatcher::has_been_set(),
        "tracing::warn! in acp and support_paths is silent without a global subscriber"
    );
}

#[test]
fn init_bootstrap_commit_notice_must_use_malvin_log_format() {
    let inc = include_str!("cli/init_cmd_mid_core.rs");
    assert!(
        inc.contains(INIT_COMMIT_NOTICE),
        "fixture must reference the init bootstrap notice string"
    );
    assert!(
        inc.contains("print_stderr_line"),
        "init bootstrap notice must go through print_stderr_line"
    );
}

#[test]
fn session_spawn_must_not_reference_cgroup_containment() {
    assert!(
        !SESSION_SPAWN_INC.contains("acp_memory_containment"),
        "session_spawn must not wire cgroup memory containment"
    );
}

#[test]
fn unix_shutdown_must_not_reference_cgroup_containment_test_api() {
    let src = include_str!("acp_session_tests/unix_shutdown.rs");
    assert!(
        !src.contains("memory_containment_cgroup_leaf_snapshot_for_tests"),
        "cgroup removal deleted memory containment; unix_shutdown must not call removed test API (breaks Linux builds)"
    );
}

#[test]
fn linux_kpop_harness_must_not_pass_removed_p_creative_flag() {
    const HARNESS: &[&str] = &[
        include_str!("../tests/common/cli_parity_tty_kpop.rs"),
        include_str!("../tests/common/cli_parity_tty.rs"),
        include_str!("../tests/cli_parity_linux_pty_a.rs"),
        include_str!("../tests/cli_parity_linux_pty_b.rs"),
    ];
    for src in HARNESS {
        assert!(
            !src.contains("--p-creative"),
            "kpop dropped --p-creative (plan A6); Linux PTY/parity harness must not pass it (clap parse error)"
        );
    }
}

#[test]
fn malvin_tracing_layer_must_forward_info_warn_and_error() {
    assert!(crate::tracing_init::malvin_log_accepts_tracing_level(
        tracing::Level::INFO
    ));
    assert!(crate::tracing_init::malvin_log_accepts_tracing_level(
        tracing::Level::WARN
    ));
    assert!(crate::tracing_init::malvin_log_accepts_tracing_level(
        tracing::Level::ERROR
    ));
    assert!(!crate::tracing_init::malvin_log_accepts_tracing_level(
        tracing::Level::DEBUG
    ));
}

#[test]
fn tracing_message_debug_field_must_not_use_rust_debug_quoting() {
    let payload = "acp message";
    assert_eq!(
        crate::tracing_init::format_debug_tracing_field("message", &payload),
        payload
    );
}

#[test]
fn malformed_rpc_timeout_env_must_use_default_and_emit_warning_on_stderr() {
    use crate::support_paths::{DEFAULT_ACP_RPC_TIMEOUT_SECS, acp_rpc_timeout_secs_from_env};
    use crate::test_stderr_capture::capture_stderr_output;
    use crate::test_utils::test_env_lock;

    crate::init_from_env();
    assert!(crate::tracing_init::malvin_log_accepts_tracing_level(
        tracing::Level::WARN
    ));
    let _guard = test_env_lock();
    let old = std::env::var("MALVIN_ACP_RPC_TIMEOUT_SECS").ok();
    unsafe {
        std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "not-a-number");
    }
    let (secs, stderr) = {
        let mut secs = DEFAULT_ACP_RPC_TIMEOUT_SECS;
        let stderr = capture_stderr_output(|| {
            secs = acp_rpc_timeout_secs_from_env();
        });
        (secs, stderr)
    };
    match &old {
        Some(v) => unsafe {
            std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", v);
        },
        None => unsafe {
            std::env::remove_var("MALVIN_ACP_RPC_TIMEOUT_SECS");
        },
    }
    assert_eq!(secs, DEFAULT_ACP_RPC_TIMEOUT_SECS);
    assert!(
        stderr.contains("[warning"),
        "malformed MALVIN_ACP_RPC_TIMEOUT_SECS must emit warning who tag on stderr"
    );
    assert!(
        stderr.contains("MALVIN_ACP_RPC_TIMEOUT_SECS"),
        "malformed env warn must mention MALVIN_ACP_RPC_TIMEOUT_SECS on stderr"
    );
}

#[test]
fn hunt_workspace_gate_failure_omits_skip_pre_checks() {
    let msg = crate::cli::format_workspace_gate_failure("malvin hunt", "`kiss check` failed");
    assert!(
        !msg.contains("--skip-pre-checks"),
        "hunt no longer supports --skip-pre-checks; message was: {msg}"
    );
    assert!(msg.contains("retry `malvin hunt`"));
}

#[test]
fn sync_review_file_for_attempt_must_not_promote_workspace_lgtm_when_artifact_empty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifact = tmp.path().join("artifact_review.md");
    let workspace = tmp.path().join("workspace_review.md");
    std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
    let synced =
        crate::review_sync::sync_review_file_for_attempt(&artifact, &workspace).expect("sync");
    assert!(
        synced.is_none(),
        "empty artifact with workspace LGTM must not be promoted (false LGTM); got {synced:?}"
    );
    assert!(
        !artifact.exists()
            || std::fs::read_to_string(&artifact)
                .expect("read artifact")
                .trim()
                .is_empty(),
        "workspace LGTM must not be copied into artifact when artifact was empty"
    );
}
