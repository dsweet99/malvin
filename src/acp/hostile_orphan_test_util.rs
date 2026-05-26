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
