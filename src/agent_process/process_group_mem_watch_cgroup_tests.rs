use super::{MemWatchHandles, watch_process_group_memory_with_rss_sampler};
use crate::agent_process::agent_cgroup::{AgentCgroup, set_active_agent_cgroup};
use crate::agent_process::sandbox_oom::OOM_REASON_CGROUP_OOM_KILL;
use crate::artifacts::create_run_artifacts_from_text;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[path = "process_group_mem_watch_test_support.rs"]
mod process_group_mem_watch_test_support;
use process_group_mem_watch_test_support::spawn_std_sleep_child_in_new_process_group;

fn fake_agent_cgroup(leader_pid: u32, oom_kills: u64) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tmp");
    std::fs::write(tmp.path().join("cgroup.procs"), format!("{leader_pid}\n")).expect("procs");
    std::fs::write(tmp.path().join("cgroup.events"), "populated 1\n").expect("events");
    std::fs::write(
        tmp.path().join("memory.events"),
        format!("oom 0\noom_kill {oom_kills}\n"),
    )
    .expect("memory.events");
    set_active_agent_cgroup(Some(AgentCgroup::new(leader_pid, tmp.path().to_path_buf())));
    tmp
}

fn handles(pgid: u32, baseline: std::collections::HashSet<u32>) -> MemWatchHandles {
    MemWatchHandles {
        reader_dead: Arc::new(AtomicBool::new(false)),
        pgid: Some(pgid),
        limit_bytes: 1,
        spawn_pid_baseline: baseline,
        run_dir: None,
    }
}

#[tokio::test]
async fn watcher_defers_memory_limit_to_kernel_when_agent_cgroup_is_active() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    let (mut child, pgid, baseline) = spawn_std_sleep_child_in_new_process_group();
    let _cgroup = fake_agent_cgroup(pgid, 0);
    let over_limit =
        watch_process_group_memory_with_rss_sampler(handles(pgid, baseline.clone()), |_, _| {
            Some(crate::agent_process::process_group_rss::MemorySample::new(
                999,
                crate::agent_process::process_group_rss::MemoryMetric::Uss,
            ))
        });
    let raced = tokio::time::timeout(std::time::Duration::from_millis(80), over_limit).await;
    let unmeasurable =
        watch_process_group_memory_with_rss_sampler(handles(pgid, baseline), |_, _| None);
    let raced_fail_closed =
        tokio::time::timeout(std::time::Duration::from_millis(80), unmeasurable).await;
    set_active_agent_cgroup(None);
    assert!(
        raced.is_err(),
        "user-space count must not kill under a cgroup"
    );
    assert!(
        raced_fail_closed.is_err(),
        "no fail-closed kill under a cgroup"
    );
    assert!(child.try_wait().expect("try_wait").is_none());
    child.kill().expect("kill sleep");
    let _ = child.wait().expect("reap sleep");
}

#[tokio::test]
async fn watcher_ends_session_and_records_marker_on_cgroup_oom_kill() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    let _saved_env = crate::test_support::test_utils::SavedEnvVars::capture(&[
        "HOME",
        crate::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION,
    ]);
    let home = tempfile::tempdir().expect("home");
    crate::test_support::test_utils::set_test_home_env(home.path());
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_run_artifacts_from_text("code", Some(tmp.path())).expect("artifacts");
    let (mut child, pgid, baseline) = spawn_std_sleep_child_in_new_process_group();
    let _cgroup = fake_agent_cgroup(pgid, 1);
    let mut h = handles(pgid, baseline);
    h.limit_bytes = u64::MAX;
    h.run_dir = Some(artifacts.run_dir.clone());
    watch_process_group_memory_with_rss_sampler(h, |_, _| {
        Some(crate::agent_process::process_group_rss::MemorySample::new(
            0,
            crate::agent_process::process_group_rss::MemoryMetric::Uss,
        ))
    })
    .await;
    set_active_agent_cgroup(None);
    let status = child.wait().expect("wait");
    assert!(
        !status.success(),
        "cgroup OOM kill must end the whole session"
    );
    let text = std::fs::read_to_string(artifacts.sandbox_oom_json_path()).expect("read");
    assert!(text.contains(OOM_REASON_CGROUP_OOM_KILL));
}
