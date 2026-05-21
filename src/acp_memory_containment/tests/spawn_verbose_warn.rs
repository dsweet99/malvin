use crate::acp_memory_containment::{
    CONTAINMENT_UNAVAILABLE_WARN, ContainmentUnavailableWarnAtSpawn,
    emit_containment_unavailable_warn_after_spawn,
};
use crate::test_stderr_capture::capture_stderr_output;

#[test]
fn containment_warn_after_spawn_suppressed_without_verbose() {
    let stderr = capture_stderr_output(|| {
        emit_containment_unavailable_warn_after_spawn(ContainmentUnavailableWarnAtSpawn {
            log_full_outgoing_prompts: false,
            containment_active: false,
        });
    });
    assert!(!stderr.contains(CONTAINMENT_UNAVAILABLE_WARN));
}

#[test]
fn containment_warn_after_spawn_emitted_with_verbose_and_inactive() {
    let stderr = capture_stderr_output(|| {
        emit_containment_unavailable_warn_after_spawn(ContainmentUnavailableWarnAtSpawn {
            log_full_outgoing_prompts: true,
            containment_active: false,
        });
    });
    assert!(stderr.contains(CONTAINMENT_UNAVAILABLE_WARN));
    assert!(stderr.contains("[warning"));
}

#[test]
fn containment_warn_after_spawn_suppressed_when_containment_active() {
    let stderr = capture_stderr_output(|| {
        emit_containment_unavailable_warn_after_spawn(ContainmentUnavailableWarnAtSpawn {
            log_full_outgoing_prompts: true,
            containment_active: true,
        });
    });
    assert!(!stderr.contains(CONTAINMENT_UNAVAILABLE_WARN));
}

fn cat_bin() -> &'static std::path::Path {
    std::path::Path::new("/bin/cat")
}

async fn stderr_during_acp_spawn(log_full_outgoing_prompts: bool) -> String {
    let tmp = tempfile::tempdir().expect("tempdir");
    let args = crate::acp::AcpSpawnArgs {
        cwd: tmp.path(),
        bin_override: Some(cat_bin()),
        api_key: Some("test-key"),
        auth_token: Some("test-token"),
        rpc_timeout: std::time::Duration::from_secs(1),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: None,
        force: false,
        tee_trace_stdout: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts,
    };
    crate::output::clear_captured_stderr_lines();
    let _ = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        crate::acp::AcpSession::spawn(args),
    )
    .await;
    crate::output::take_captured_stderr_lines().join("")
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn acp_session_spawn_suppresses_containment_warn_without_verbose() {
    crate::init_from_env();
    let stderr = stderr_during_acp_spawn(false).await;
    assert!(!stderr.contains(CONTAINMENT_UNAVAILABLE_WARN));
}

#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn acp_session_spawn_emits_containment_warn_with_verbose_and_inactive_containment() {
    crate::init_from_env();
    let stderr = stderr_during_acp_spawn(true).await;
    assert!(stderr.contains(CONTAINMENT_UNAVAILABLE_WARN));
    assert!(stderr.contains("[warning"));
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn acp_session_spawn_containment_warn_when_cgroups_unavailable() {
    if crate::acp_memory_containment::test_support::writable_cgroups_on_host() {
        return;
    }
    crate::init_from_env();
    let without = stderr_during_acp_spawn(false).await;
    assert!(!without.contains(CONTAINMENT_UNAVAILABLE_WARN));
    let with = stderr_during_acp_spawn(true).await;
    assert!(with.contains(CONTAINMENT_UNAVAILABLE_WARN));
}
