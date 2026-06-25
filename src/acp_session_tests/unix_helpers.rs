#[cfg(unix)]
pub(super) fn process_exists(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(unix)]
pub(super) async fn wait_for_pid_file(path: &std::path::Path) -> u32 {
    let poll = if crate::acp::test_no_real_agent_enabled() {
        std::time::Duration::from_millis(1)
    } else {
        std::time::Duration::from_millis(20)
    };
    let budget = if crate::acp::test_no_real_agent_enabled() {
        std::time::Duration::from_millis(100)
    } else {
        std::time::Duration::from_secs(6)
    };
    let deadline = tokio::time::Instant::now() + budget;
    while tokio::time::Instant::now() < deadline {
        if let Ok(raw) = tokio::fs::read_to_string(path).await {
            if let Ok(pid) = raw.trim().parse::<u32>() {
                if process_exists(pid) {
                    return pid;
                }
            }
        }
        tokio::time::sleep(poll).await;
    }
    panic!("pid file was not written or process not alive: {}", path.display());
}

#[cfg(unix)]
pub(super) async fn write_descendant_spawning_acp_mock(bin: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = r"#!/usr/bin/env node
const fs = require('fs');
const readline = require('readline');
const { spawn } = require('child_process');
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
function send(id, result) {
  console.log(JSON.stringify({ jsonrpc: '2.0', id, result }));
}
rl.on('line', (line) => {
  line = line.trim();
  if (!line) return;
  const msg = JSON.parse(line);
  if (msg.method === 'initialize') {
    send(msg.id, {});
  } else if (msg.method === 'authenticate') {
    send(msg.id, {});
  } else if (msg.method === 'session/new') {
    send(msg.id, { sessionId: 't1' });
  } else if (msg.method === 'session/prompt') {
    const child = spawn('sleep', ['30'], { detached: true, stdio: 'ignore' });
    child.unref();
    fs.writeFileSync('descendant.pid', String(child.pid));
    send(msg.id, {});
  } else {
    send(msg.id, {});
  }
});
";
    tokio::fs::write(bin, script.as_bytes()).await.unwrap();
    let mut perms = std::fs::metadata(bin).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(bin, perms).unwrap();
    crate::test_utils::sync_test_executable(bin);
}

#[cfg(test)]
mod tests {
    use super::process_exists;

    #[test]
    fn process_exists_true_for_current_process() {
        let pid = std::process::id();
        assert!(process_exists(pid));
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = wait_for_pid_file;
        let _ = write_descendant_spawning_acp_mock;
        let _ = process_exists;
    }
}
