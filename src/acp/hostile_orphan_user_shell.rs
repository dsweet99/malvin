//! User-shell cooperator fixtures for coincidental-daemon regression tests.

use std::io::Write;
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use super::process_alive;
use super::read_orphan_pid;
use super::wait_for_init_reparent;

/// Spawns a long-lived cooperator process (simulating the user's shell) that forks
/// double-fork daemons on stdin command `DAEMON <pidfile>`.
pub fn spawn_user_shell_cooperator() -> (Child, ChildStdin) {
    let script = r"import os,sys,time
while True:
 line=sys.stdin.readline()
 if not line:
  break
 parts=line.strip().split()
 if len(parts)>=2 and parts[0]=='DAEMON':
  pidfile=parts[1]
  pid=os.fork()
  if pid==0:
   os.setsid()
   g=os.fork()
   if g==0:
    open(pidfile,'w').write(str(os.getpid()))
    os.execvp('sleep',['sleep','120'])
   time.sleep(0.2)
   os._exit(0)
  os.waitpid(pid,0)
";
    let mut cmd = Command::new("python3");
    cmd.arg("-c").arg(script);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let mut child = cmd.spawn().expect("spawn user shell cooperator");
    let stdin = child.stdin.take().expect("stdin");
    (child, stdin)
}

/// Double-fork daemon spawned from a baseline user-shell cooperator (not malvin/agent).
pub fn spawn_user_coincidental_daemon(user_shell_stdin: &mut ChildStdin, orphan_pid_file: &Path) {
    writeln!(
        user_shell_stdin,
        "DAEMON {}",
        orphan_pid_file.display()
    )
    .expect("request user daemon spawn");
    user_shell_stdin.flush().expect("flush");
}

/// Spawns an isolated-PG agent sleep child for sandbox teardown contract tests.
pub fn spawn_isolated_agent_sleep() -> (Child, u32) {
    use std::os::unix::process::CommandExt;
    let mut agent = Command::new("sleep");
    agent.arg("120").process_group(0);
    let child = agent.spawn().expect("spawn agent");
    let pgid = child.id();
    (child, pgid)
}

/// Spawns a user coincidental daemon and waits until it reparents to init.
pub async fn setup_user_init_reparented_daemon(
    user_shell_stdin: &mut ChildStdin,
    orphan_pid_file: &Path,
) -> u32 {
    spawn_user_coincidental_daemon(user_shell_stdin, orphan_pid_file);
    let pid = read_orphan_pid(orphan_pid_file, None).await;
    wait_for_init_reparent(pid).await;
    assert!(
        process_alive(pid),
        "setup: user daemon should be running"
    );
    pid
}

/// Cleans up processes spawned by [`user_coincidental_init_orphan_survives_agent_teardown`].
pub fn cleanup_user_coincidental_test(user_daemon_pid: u32, mut user_shell: Child, mut agent_child: Child) {
    let _ = Command::new("kill")
        .args(["-9", &user_daemon_pid.to_string()])
        .status();
}

#[cfg(test)]
#[path = "hostile_orphan_user_shell_test.rs"]
mod hostile_orphan_user_shell_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = cleanup_user_coincidental_test;
        let _ = setup_user_init_reparented_daemon;
        let _ = spawn_isolated_agent_sleep;
        let _ = spawn_user_coincidental_daemon;
        let _ = spawn_user_shell_cooperator;
    }
}
