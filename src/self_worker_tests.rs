use super::{
    Worker, attached_command, detached_command, invocation, is_worker_cmdline, parent_disconnected,
    spawn_detached, usage,
};
use std::ffi::{OsStr, OsString};

fn os(args: &[&str]) -> std::vec::IntoIter<OsString> {
    args.iter()
        .map(OsString::from)
        .collect::<Vec<_>>()
        .into_iter()
}

fn env_of(command: &std::process::Command, key: &str) -> Option<Option<OsString>> {
    command
        .get_envs()
        .find(|(k, _)| *k == OsStr::new(key))
        .map(|(_, v)| v.map(OsStr::to_os_string))
}

#[test]
fn workers_share_markers_dispatch_and_isolation() {
    assert!(invocation(os(&[])).is_none());
    assert!(invocation(os(&["--help"])).is_none());
    assert!(invocation(os(&["REQUEST", "__summary-worker"])).is_none());
    for worker in Worker::ALL {
        assert!(worker.marker().starts_with("__"));
        let (found, rest) = invocation(os(&[worker.marker(), "a", "b"])).unwrap();
        assert_eq!(found, worker);
        assert_eq!(rest, vec![OsString::from("a"), OsString::from("b")]);
        assert!(usage(worker, "X").ends_with(&format!("{} X", worker.marker())));
    }
    let log = tempfile::tempfile().unwrap();
    let attached = attached_command(Worker::Summary, ["plan.md"], log).unwrap();
    let detached = detached_command(Worker::HerdrLinger, ["sock", "pane", "1"]).unwrap();
    for command in [attached.as_std(), &detached] {
        assert_eq!(env_of(command, "MALVIN_NO_HERDR"), Some(Some("1".into())));
        for var in crate::herdr::HERDR_ENV_VARS {
            assert_eq!(env_of(command, var), Some(None), "{var} must be removed");
        }
    }
    let args: Vec<_> = detached.get_args().collect();
    assert_eq!(args, ["__herdr-linger", "sock", "pane", "1"]);
    assert!(detached_command(Worker::LocalLlmManager, ["x"]).is_ok());
    assert!(detached_command(Worker::FactAgent, ["x"]).is_err());
    let log = tempfile::tempfile().unwrap();
    assert!(attached_command(Worker::LocalLlmManager, ["x"], log).is_err());
    let _ = parent_disconnected;
    detached_workers_leave_no_zombie_child();
    assert!(is_worker_cmdline(
        b"/usr/bin/malvin\0__fact-agent-worker\0x\0"
    ));
    assert!(!is_worker_cmdline(
        b"/usr/bin/malvin\0admin\0__fact-agent-worker\0"
    ));
    assert!(!is_worker_cmdline(b""));
    summary_worker_round_trips_through_the_shared_marker();
}

fn summary_worker_round_trips_through_the_shared_marker() {
    use crate::task_summary::{summary_worker_command, summary_worker_request};
    let dir = tempfile::tempdir().unwrap();
    let artifacts = crate::artifacts::RunArtifacts {
        run_dir: dir.path().to_path_buf(),
        plan_path: dir.path().join("plan.md"),
        work_dir: dir.path().to_path_buf(),
    };
    let command = summary_worker_command(&artifacts).unwrap();
    assert_eq!(command.as_std().get_current_dir(), Some(dir.path()));
    let args = command.as_std().get_args().map(OsString::from);
    let (worker, rest) = invocation(args).unwrap();
    assert_eq!(worker, Worker::Summary);
    assert_eq!(summary_worker_request(rest).unwrap(), artifacts.plan_path);
    assert!(
        summary_worker_request(vec![])
            .unwrap_err()
            .contains("usage")
    );
    assert!(summary_worker_request(vec!["a".into(), "b".into()]).is_err());
}

fn detached_workers_leave_no_zombie_child() {
    for worker in [Worker::LocalLlmManager, Worker::HerdrLinger] {
        let pid = spawn_detached(worker, ["--exact", "no_such_test"]).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while std::path::Path::new(&format!("/proc/{pid}")).exists() {
            assert!(
                std::time::Instant::now() < deadline,
                "{} child {pid} was never reaped",
                worker.marker()
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
