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
    for _ in 0..100 {
        if let Ok(raw) = tokio::fs::read_to_string(path).await {
            if let Ok(pid) = raw.trim().parse::<u32>() {
                return pid;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("pid file was not written: {}", path.display());
}

#[cfg(unix)]
pub(super) async fn write_descendant_spawning_acp_mock(bin: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = r"#!/usr/bin/env node
const readline = require('readline');
const { spawnSync } = require('child_process');
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
    spawnSync('/bin/sh', ['-c', 'nohup sleep 30 >/dev/null 2>&1 & echo $! > descendant.pid'], { cwd: process.cwd() });
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
