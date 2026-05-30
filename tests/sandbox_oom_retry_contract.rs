//! Adversarial tests: OOM retry detection must not grep agent transcripts.

use malvin::{
    create_kpop_run_artifacts, format_current_state, gate_iteration_oom_killed,
};

#[test]
fn kpop_log_oom_prose_does_not_trigger_retry_reason() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    std::fs::write(
        artifacts.log_path("kpop"),
        "Our test output exceeded memory limit in the benchmark harness\n",
    )
    .expect("write");
    let line = format_current_state(tmp.path(), Some(2), Some(&artifacts));
    assert!(
        !line.contains("OOM"),
        "agent transcript prose must not infer malvin OOM kills: {line}"
    );
    assert!(!gate_iteration_oom_killed(&artifacts, 1));
}

#[test]
fn sandbox_oom_marker_is_iteration_scoped() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("code", Some(tmp.path())).expect("artifacts");
    malvin::record_sandbox_oom_kill(
        &artifacts.run_dir,
        malvin::SandboxOomKillRecord::from_facts(
            3,
            malvin::SandboxOomKillFacts {
                reason: malvin::OOM_REASON_MEMORY_LIMIT,
                rss_bytes: Some(1),
                limit_bytes: 1,
                pgid: 1,
            },
        ),
    )
    .expect("write");
    assert!(!gate_iteration_oom_killed(&artifacts, 1));
    assert!(!gate_iteration_oom_killed(&artifacts, 2));
    assert!(gate_iteration_oom_killed(&artifacts, 3));
}
