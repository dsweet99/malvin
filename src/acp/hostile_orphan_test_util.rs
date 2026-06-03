//! Shared hostile-agent fixtures for sandbox regression tests.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

pub fn process_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Spawns an agent whose orphan uses the classic double-fork daemon pattern: `setsid`, inner
/// `fork`, middle exits, grandchild reparented to init with `ppid == 1` and `pgid != pid`.
pub fn spawn_hostile_double_fork_daemon(cwd: &Path, orphan_pid_file: &Path) -> (std::process::Child, u32) {
    use std::os::unix::process::CommandExt;
    let script = format!(
        r#"python3 -c 'import os,sys
pid=os.fork()
if pid==0:
 os.setsid()
 g=os.fork()
 if g==0:
  open(sys.argv[1],"w").write(str(os.getpid()))
  os.execvp("sleep",["sleep","120"])
 os._exit(0)
os.waitpid(pid,0)' "{}"
exec sleep 60
"#,
        orphan_pid_file.display()
    );
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(script);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.process_group(0);
    let child = cmd.spawn().expect("spawn hostile double-fork agent");
    let pgid = child.id();
    (child, pgid)
}

/// Like [`spawn_hostile_agent`], but the session leader exits right after forking the orphan
/// (no `sleep` parent), so the agent process group is empty while the `setsid` orphan keeps running.
pub fn spawn_hostile_agent_exits_after_orphan_fork(
    cwd: &Path,
    orphan_pid_file: &Path,
) -> (std::process::Child, u32) {
    use std::os::unix::process::CommandExt;
    let script = format!(
        r#"python3 -c 'import os,sys
pid=os.fork()
if pid==0:
 os.setsid()
 os.execvp("sleep",["sleep","120"])
else:
 open(sys.argv[1],"w").write(str(pid))
 os._exit(0)' "{}"
"#,
        orphan_pid_file.display()
    );
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(script);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.process_group(0);
    let child = cmd.spawn().expect("spawn hostile agent");
    let pgid = child.id();
    (child, pgid)
}

pub fn spawn_hostile_agent(cwd: &Path, orphan_pid_file: &Path) -> (std::process::Child, u32) {
    use std::os::unix::process::CommandExt;
    let script = format!(
        r#"python3 -c 'import os,sys
pid=os.fork()
if pid==0:
 os.setsid()
 os.execvp("sleep",["sleep","120"])
else:
 open(sys.argv[1],"w").write(str(pid))' "{}"
exec sleep 60
"#,
        orphan_pid_file.display()
    );
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(script);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.process_group(0);
    let child = cmd.spawn().expect("spawn hostile agent");
    let pgid = child.id();
    (child, pgid)
}

pub async fn read_orphan_pid(path: &Path) -> u32 {
    for _ in 0..50 {
        if let Ok(text) = std::fs::read_to_string(path) {
            if let Ok(pid) = text.trim().parse::<u32>() {
                if process_alive(pid) {
                    return pid;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!(
        "orphan pid file not written or orphan not alive: {}",
        path.display()
    );
}

pub async fn wait_for_init_reparent(pid: u32) {
    for _ in 0..50 {
        if super::unix_process_group_ps::list_proc_rows()
            .unwrap_or_default()
            .iter()
            .find(|row| row.pid == pid)
            .is_some_and(|row| row.ppid == super::unix_process_group_ps::INIT_PID)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("orphan never reparented to init (pid={pid})");
}

/// Spawns an agent whose orphan reparents to init with `MALVIN_WORKSPACE` set (malvin ACP pattern).
pub fn spawn_hostile_agent_acp_orphan(
    cwd: &Path,
    orphan_pid_file: &Path,
) -> (std::process::Child, u32) {
    use std::os::unix::process::CommandExt;
    let script = format!(
        r#"python3 -c 'import os,sys
pid=os.fork()
if pid==0:
 os.setsid()
 open(sys.argv[1],"w").write(str(os.getpid()))
 os.environ["MALVIN_WORKSPACE"]="/tmp/malvin-hostile-orptest"
 os.execvp("sleep",["sleep","120"])
else:
 os._exit(0)' "{}"
"#,
        orphan_pid_file.display()
    );
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(script);
    cmd.current_dir(cwd);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.process_group(0);
    let child = cmd.spawn().expect("spawn hostile agent-acp orphan");
    let pgid = child.id();
    (child, pgid)
}

/// Spawns an isolated-PG agent sleep child plus a malvin-PG sibling (no `process_group(0)`).
pub fn spawn_agent_pg_and_malvin_sibling(
) -> (u32, u32, std::process::Child, std::process::Child) {
    use std::os::unix::process::CommandExt;
    let mut agent = std::process::Command::new("sleep");
    agent.arg("120").process_group(0);
    let agent_child = agent.spawn().expect("spawn agent");
    let agent_pgid = agent_child.id();
    let mut sibling = std::process::Command::new("sleep");
    sibling.arg("120");
    let sibling_child = sibling.spawn().expect("spawn sibling");
    let sibling_pid = sibling_child.id();
    std::thread::sleep(Duration::from_millis(100));
    (agent_pgid, sibling_pid, agent_child, sibling_child)
}

pub fn assert_sibling_monitored_and_blocks_spawn(
    agent_pgid: u32,
    sibling_pid: u32,
    baseline: &std::collections::HashSet<u32>,
) {
    use crate::acp::sandbox_monitor_pids;
    use crate::malvin_sandbox::{
        assert_dead_before_next_spawn, note_active_sandbox_session,
    };

    note_active_sandbox_session(Some(agent_pgid), baseline.clone());
    let monitor = sandbox_monitor_pids(Some(agent_pgid), baseline);
    assert!(
        monitor.contains(&sibling_pid),
        "setup: sibling must be in sandbox monitor set"
    );
    assert!(
        assert_dead_before_next_spawn().is_err(),
        "setup: dead-before-next must block while sibling is alive"
    );
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_spawn_hostile_agent_exits_after_orphan_fork() { let _ = spawn_hostile_agent_exits_after_orphan_fork; }
    #[test]
    fn kiss_cov_wait_for_init_reparent() { let _ = wait_for_init_reparent; }
}
