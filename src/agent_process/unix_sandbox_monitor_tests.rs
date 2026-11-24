use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::time::{Duration, Instant};

use super::sandbox_monitor_pids;

struct SpawnedTree {
    root: std::process::Child,
    grandchild: u32,
}

impl Drop for SpawnedTree {
    fn drop(&mut self) {
        crate::agent_process::unix_process_group_ps::signal_pid(self.grandchild, libc::SIGKILL);
        let _ = self.root.kill();
        let _ = self.root.wait();
    }
}

fn spawn_tree_as(dir: &std::path::Path, script_name: &str) -> SpawnedTree {
    let script = dir.join(script_name);
    std::fs::write(&script, "sleep 30 &\necho $! > \"$0.child\"\nwait\n").expect("write script");
    let root = std::process::Command::new("sh")
        .arg(script_name)
        .current_dir(dir)
        .process_group(0)
        .spawn()
        .expect("spawn sh");
    let pid_file = dir.join(format!("{script_name}.child"));
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(pid) = std::fs::read_to_string(&pid_file)
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
        {
            return SpawnedTree {
                root,
                grandchild: pid,
            };
        }
        assert!(Instant::now() < deadline, "child pid never written");
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub(crate) fn sandbox_monitor_excludes_malvin_workers_and_their_children() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::clear_session_spawn_affiliation();
    let baseline: HashSet<u32> = HashSet::from([1]);
    let dir = tempfile::tempdir().expect("tempdir");
    let worker = spawn_tree_as(dir.path(), crate::self_worker::Worker::FactAgent.marker());
    let control = spawn_tree_as(dir.path(), "plain_child");
    let control_pid = control.root.id();
    let worker_pid = worker.root.id();
    let monitored = sandbox_monitor_pids(Some(control_pid), &baseline);
    crate::agent_process::clear_session_spawn_affiliation();
    assert!(
        monitored.contains(&control_pid),
        "agent group must be monitored"
    );
    assert!(monitored.contains(&control.grandchild));
    assert!(
        !monitored.contains(&worker_pid),
        "worker {worker_pid} counted as agent"
    );
    assert!(
        !monitored.contains(&worker.grandchild),
        "worker child {} counted as agent",
        worker.grandchild
    );
    assert!(
        !monitored.contains(&std::process::id()),
        "malvin itself is not part of the agent"
    );
}
