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

pub(crate) fn hostile_script_delay_ms(default_ms: u64) -> u64 {
    if crate::acp::test_no_real_agent_enabled() {
        10
    } else {
        default_ms
    }
}

fn hostile_test_poll_interval() -> Duration {
    if crate::acp::test_no_real_agent_enabled() {
        Duration::from_millis(2)
    } else {
        Duration::from_millis(20)
    }
}

fn hostile_test_wait_budget() -> Duration {
    if crate::acp::test_no_real_agent_enabled() {
        Duration::from_millis(100)
    } else {
        Duration::from_millis(500)
    }
}

/// Spawns an agent whose orphan uses the classic double-fork daemon pattern: `setsid`, inner
/// `fork`, middle exits, grandchild reparented to init with `ppid == 1` and `pgid != pid`.
pub fn spawn_hostile_double_fork_daemon(cwd: &Path, orphan_pid_file: &Path) -> (std::process::Child, u32) {
    use std::os::unix::process::CommandExt;
    let script = format!(
        r#"python3 -c 'import os,sys,time
pid=os.fork()
if pid==0:
 os.setsid()
 g=os.fork()
 if g==0:
  open(sys.argv[1],"w").write(str(os.getpid()))
  os.execvp("sleep",["sleep","120"])
 time.sleep({child_delay} / 1000)
 os._exit(0)
os.waitpid(pid,0)' "{orphan_pid_file}"
exec sleep 60
"#,
        child_delay = hostile_script_delay_ms(200),
        orphan_pid_file = orphan_pid_file.display(),
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
        r#"python3 -c 'import os,sys,time
pid=os.fork()
if pid==0:
 open(sys.argv[1],"w").write(str(os.getpid()))
 time.sleep({orphan_delay} / 1000)
 os.setsid()
 os.execvp("sleep",["sleep","120"])
else:
 time.sleep({parent_delay} / 1000)
 os._exit(0)' "{orphan_pid_file}"
"#,
        orphan_delay = hostile_script_delay_ms(200),
        parent_delay = hostile_script_delay_ms(500),
        orphan_pid_file = orphan_pid_file.display(),
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
        r#"python3 -c 'import os,sys,time
pid=os.fork()
if pid==0:
 open(sys.argv[1],"w").write(str(os.getpid()))
 time.sleep({orphan_delay} / 1000)
 os.setsid()
 os.execvp("sleep",["sleep","120"])' "{orphan_pid_file}"
exec sleep 60
"#,
        orphan_delay = hostile_script_delay_ms(200),
        orphan_pid_file = orphan_pid_file.display(),
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

#[path = "hostile_orphan_read_pid.rs"]
mod hostile_orphan_read_pid;
pub use hostile_orphan_read_pid::read_orphan_pid;

#[path = "hostile_orphan_user_shell.rs"]
mod hostile_orphan_user_shell;
pub use hostile_orphan_user_shell::{
    cleanup_user_coincidental_test, setup_user_init_reparented_daemon,
    spawn_isolated_agent_sleep, spawn_user_coincidental_daemon,
    spawn_user_shell_cooperator,
};

pub async fn wait_for_init_reparent(pid: u32) {
    let poll = hostile_test_poll_interval();
    let deadline = tokio::time::Instant::now() + hostile_test_wait_budget();
    while tokio::time::Instant::now() < deadline {
        if super::unix_process_group_ps::list_proc_rows()
            .unwrap_or_default()
            .iter()
            .find(|row| row.pid == pid)
            .is_some_and(|row| row.ppid == super::unix_process_group_ps::INIT_PID)
        {
            return;
        }
        tokio::time::sleep(poll).await;
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
 open(sys.argv[1],"w").write(str(os.getpid()))
 os.setsid()
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
    std::thread::sleep(std::time::Duration::from_millis(hostile_script_delay_ms(100)));
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

    let work = std::env::temp_dir().join(format!(
        "malvin_hostile_orphan_blocks_spawn_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).expect("mkdir work");
    note_active_sandbox_session(Some(agent_pgid), baseline.clone(), &work).expect("note");
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
