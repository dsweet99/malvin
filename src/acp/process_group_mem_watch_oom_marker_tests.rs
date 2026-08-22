//! OOM-marker tests split out of `process_group_mem_watch_tests.rs` (kiss
//! lines-per-file limit).

use super::record_sandbox_oom_marker;
use crate::artifacts::create_run_artifacts_from_text;
use crate::sandbox_oom::{
    OOM_REASON_MEASUREMENT_FAIL_CLOSED, SandboxOomKillFacts,
    gate_iteration_oom_killed,
};

#[cfg(unix)]
#[test]
fn record_sandbox_oom_marker_noops_without_run_dir() {
    record_sandbox_oom_marker(
        None,
        SandboxOomKillFacts {
            reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
            rss_bytes: None,
            limit_bytes: 1,
            pgid: 1,
        },
    );
}
#[cfg(unix)]
#[test]
fn record_sandbox_oom_marker_writes_when_gate_iteration_unset() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            create_run_artifacts_from_text("code", Some(tmp.path())).expect("artifacts");
        crate::gate_loop_session::set_active_gate_iteration(None);
        record_sandbox_oom_marker(
            Some(&artifacts.run_dir),
            SandboxOomKillFacts {
                reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
                rss_bytes: None,
                limit_bytes: 1,
                pgid: 1,
            },
        );
        assert!(gate_iteration_oom_killed(&artifacts, 0));
        let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
        assert!(text.contains(OOM_REASON_MEASUREMENT_FAIL_CLOSED));
    });
}
#[cfg(unix)]
#[test]
fn record_sandbox_oom_marker_writes_fail_closed_reason() {
    let _lock = crate::test_utils::test_env_lock();
    let _saved_env = crate::test_utils::SavedEnvVars::capture(&[
        "HOME",
        crate::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION,
    ]);
    let home = tempfile::tempdir().expect("home");
    crate::test_utils::set_test_home_env(home.path());
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_run_artifacts_from_text("code", Some(tmp.path())).expect("artifacts");
    crate::gate_loop_session::set_active_gate_iteration(Some(1));
    record_sandbox_oom_marker(
        Some(&artifacts.run_dir),
        SandboxOomKillFacts {
            reason: OOM_REASON_MEASUREMENT_FAIL_CLOSED,
            rss_bytes: None,
            limit_bytes: 512,
            pgid: 7,
        },
    );
    crate::gate_loop_session::set_active_gate_iteration(None);
    assert!(gate_iteration_oom_killed(&artifacts, 1));
    let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
    assert!(text.contains(OOM_REASON_MEASUREMENT_FAIL_CLOSED));
}
