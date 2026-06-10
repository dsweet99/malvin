//! Test helpers for ACP mocks (stdio JSON-RPC test doubles).
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

use std::path::Path;
use std::sync::Mutex;

/// Script body for minimal stdio `agent acp` test doubles (JSON-RPC handlers only).
pub const ACP_MOCK_JSONRPC_LOOP_JS: &str = r#"const readline = require('readline');
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on('line', (line) => {
  line = line.trim();
  if (!line) return;
  let msg;
  try { msg = JSON.parse(line); } catch (e) { return; }
  const mid = msg.method;
  const rid = msg.id;
  if (mid === 'initialize') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'authenticate') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { sessionId: 't1' } }));
  } else if (mid === 'session/cancel') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/prompt') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

/// Like [`ACP_MOCK_JSONRPC_LOOP_JS`] but `session/prompt` returns a JSON-RPC error.
const ACP_MOCK_JSONRPC_PROMPT_FAILS_JS: &str = r#"const readline = require('readline');
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on('line', (line) => {
  line = line.trim();
  if (!line) return;
  let msg;
  try { msg = JSON.parse(line); } catch (e) { return; }
  const mid = msg.method;
  const rid = msg.id;
  if (mid === 'initialize') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'authenticate') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { sessionId: 't1' } }));
  } else if (mid === 'session/cancel') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/prompt') {
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      id: rid,
      error: { code: -32000, message: 'mock prompt rpc error' },
    }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

#[path = "test_utils_acp_rpc_trace.rs"] mod acp_rpc_trace;
pub use acp_rpc_trace::write_acp_jsonrpc_mock_rpc_trace;

/// `session/cancel` returns JSON-RPC error; `session/prompt` blocks on a release file so tests can
/// prove prompt/cancel interleaving without paying real sleep time.
#[cfg(unix)]
const ACP_MOCK_CANCEL_ERR_SLOW_PROMPT_JS: &str = r#"const fs = require('fs');
const path = require('path');
const readline = require('readline');
const ROOT = __dirname;
const RELEASE = path.join(ROOT, 'allow_prompt_complete');
function bumpPromptHit() {
  fs.appendFileSync(path.join(ROOT, 'prompt_hits'), 'x\n');
}
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on('line', (line) => {
  line = line.trim();
  if (!line) return;
  let msg;
  try { msg = JSON.parse(line); } catch (e) { return; }
  const mid = msg.method;
  const rid = msg.id;
  if (mid === 'initialize') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'authenticate') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { sessionId: 't1' } }));
  } else if (mid === 'session/cancel') {
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      id: rid,
      error: { code: -32000, message: 'mock cancel failed' },
    }));
  } else if (mid === 'session/prompt') {
    bumpPromptHit();
    const myRid = rid;
    const iv = setInterval(() => {
      if (fs.existsSync(RELEASE)) {
        clearInterval(iv);
        console.log(JSON.stringify({ jsonrpc: '2.0', id: myRid, result: { stopReason: 'end' } }));
      }
    }, 1);
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

/// Process environment is global; hold this lock around any `set_var` / `remove_var` in tests.
pub static MALVIN_TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Best-effort `fsync` after writing a test executable (reduces `ETXTBSY` races).
pub fn sync_test_executable(path: &Path) {
    if let Ok(f) = std::fs::File::open(path) {
        let _ = f.sync_all();
    }
}

#[cfg(unix)]
fn chmod755_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).unwrap();
}

fn acp_mock_start_log_header(start_log: Option<&Path>) -> String {
    start_log
        .map(|p| {
            // `{:?}` yields a valid JavaScript double-quoted string literal (escaped).
            format!(
                "require('fs').appendFileSync({:?}, 'x');\n",
                p.display().to_string()
            )
        })
        .unwrap_or_default()
}

pub(crate) async fn write_acp_mock_script_with_optional_log(
    path: &Path,
    start_log: Option<&Path>,
    script_tail: &str,
) {
    let body = format!(
        "#!/usr/bin/env node\n{}{}",
        acp_mock_start_log_header(start_log),
        script_tail
    );
    tokio::fs::write(path, body.as_bytes()).await.unwrap();
    #[cfg(unix)]
    chmod755_executable(path);
    sync_test_executable(path);
}

/// Writable fake `agent acp` (Unix script with optional startup log hook for spawn races).
pub async fn write_acp_jsonrpc_mock_executable(path: &Path, start_log: Option<&Path>) {
    write_acp_mock_script_with_optional_log(path, start_log, ACP_MOCK_JSONRPC_LOOP_JS).await;
}

/// Successful handshake; `session/prompt` responds with a JSON-RPC error.
pub async fn write_acp_jsonrpc_mock_executable_prompt_fails(path: &Path, start_log: Option<&Path>) {
    write_acp_mock_script_with_optional_log(path, start_log, ACP_MOCK_JSONRPC_PROMPT_FAILS_JS)
        .await;
}

/// Cancel error + slow prompt (see George integration tests).
#[cfg(unix)]
pub async fn write_acp_jsonrpc_mock_cancel_err_slow_prompt(path: &Path) {
    let body = format!(
        "#!/usr/bin/env node\n{}",
        ACP_MOCK_CANCEL_ERR_SLOW_PROMPT_JS
    );
    tokio::fs::write(path, body.as_bytes()).await.unwrap();
    chmod755_executable(path);
    sync_test_executable(path);
}

pub fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    MALVIN_TEST_ENV_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
pub fn with_cwd<T>(cwd: &std::path::Path, f: impl FnOnce() -> T) -> T {
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(cwd).expect("chdir");
    let out = f();
    std::env::set_current_dir(old).expect("restore");
    out
}

#[cfg(test)]
#[path = "test_isolated_home.rs"]
mod isolated_home;

#[cfg(test)]
pub use isolated_home::with_isolated_home;

#[cfg(test)]
pub fn empty_session_dotfile_backups(work: &Path) -> crate::artifacts::SessionDotfileBackups {
    crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot session dotfiles")
}

#[cfg(test)]
mod start_log_header_tests {
    use std::process::Command;

    use super::acp_mock_start_log_header;

    #[test]
    fn acp_mock_start_log_header_is_valid_node_syntax_when_path_set() {
        let tmp = tempfile::tempdir().unwrap();
        let log = tmp.path().join("spawn-race.log");
        let header = acp_mock_start_log_header(Some(log.as_path()));
        assert!(!header.is_empty());
        let script = tmp.path().join("syntax-check.js");
        std::fs::write(&script, format!("#!/usr/bin/env node\n{header}")).unwrap();
        let ok = Command::new("node")
            .args(["--check", script.to_str().expect("utf8 path")])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(
            ok,
            "start log hook must be valid JavaScript (node --check failed):\n{header}"
        );
    }
}
