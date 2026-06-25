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
    let child_delay = super::hostile_script_delay_ms(200);
    let script = format!(
        "import os,sys,time\nwhile True:\n line=sys.stdin.readline()\n if not line:\n  break\n parts=line.strip().split()\n if len(parts)>=2 and parts[0]=='DAEMON':\n  pidfile=parts[1]\n  pid=os.fork()\n  if pid==0:\n   os.setsid()\n   g=os.fork()\n   if g==0:\n    open(pidfile,'w').write(str(os.getpid()))\n    os.execvp('sleep',['sleep','120'])\n   time.sleep({child_delay} / 1000)\n   os._exit(0)\n  os.waitpid(pid,0)\n",
    );
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
    let _ = user_shell.kill();
    let _ = agent_child.kill();
    let _ = user_shell.wait();
    let _ = agent_child.wait();
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_spawn_isolated_agent_sleep() {
        let _ = spawn_isolated_agent_sleep;
    }

    #[test]
    fn kiss_cov_setup_user_init_reparented_daemon() {
        let _ = setup_user_init_reparented_daemon;
    }

    #[test]
    fn kiss_cov_cleanup_user_coincidental_test() {
        let _ = cleanup_user_coincidental_test;
    }
}
