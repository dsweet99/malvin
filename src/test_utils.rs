//! Test helpers for ACP mocks (mirrors `Projects/george/src/test_utils.rs` patterns).
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

use std::path::Path;
use std::sync::Mutex;

/// Script body for minimal stdio `agent acp` test doubles (JSON-RPC handlers only).
pub const ACP_MOCK_JSONRPC_LOOP_PY: &str = r#"import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    mid = msg.get("method")
    rid = msg.get("id")
    if mid == "initialize":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "authenticate":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "session/new":
        print(
            json.dumps({"jsonrpc": "2.0", "id": rid, "result": {"sessionId": "t1"}}),
            flush=True,
        )
    elif mid == "session/cancel":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "session/prompt":
        print(
            json.dumps({"jsonrpc": "2.0", "id": rid, "result": {"stopReason": "end"}}),
            flush=True,
        )
    elif rid is not None:
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
"#;

/// Like [`ACP_MOCK_JSONRPC_LOOP_PY`] but `session/prompt` returns a JSON-RPC error.
const ACP_MOCK_JSONRPC_PROMPT_FAILS_PY: &str = r#"import sys, json
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    mid = msg.get("method")
    rid = msg.get("id")
    if mid == "initialize":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "authenticate":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "session/new":
        print(
            json.dumps({"jsonrpc": "2.0", "id": rid, "result": {"sessionId": "t1"}}),
            flush=True,
        )
    elif mid == "session/cancel":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "session/prompt":
        print(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": rid,
                    "error": {"code": -32000, "message": "mock prompt rpc error"},
                }
            ),
            flush=True,
        )
    elif rid is not None:
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
"#;

/// `session/cancel` returns JSON-RPC error; `session/prompt` is delayed on a thread.
#[cfg(unix)]
const ACP_MOCK_CANCEL_ERR_SLOW_PROMPT_PY: &str = r#"import sys, json, time, threading
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def bump_prompt_hit():
    (ROOT / "prompt_hits").open("a").write("x\n")


for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    mid = msg.get("method")
    rid = msg.get("id")
    if mid == "initialize":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "authenticate":
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
    elif mid == "session/new":
        print(
            json.dumps({"jsonrpc": "2.0", "id": rid, "result": {"sessionId": "t1"}}),
            flush=True,
        )
    elif mid == "session/cancel":
        print(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": rid,
                    "error": {"code": -32000, "message": "mock cancel failed"},
                }
            ),
            flush=True,
        )
    elif mid == "session/prompt":
        bump_prompt_hit()

        def slow_reply(prompt_id):
            time.sleep(0.35)
            print(
                json.dumps(
                    {
                        "jsonrpc": "2.0",
                        "id": prompt_id,
                        "result": {"stopReason": "end"},
                    }
                ),
                flush=True,
            )

        threading.Thread(target=slow_reply, args=(rid,), daemon=True).start()
    elif rid is not None:
        print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": {}}), flush=True)
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
            format!(
                "open(r'{}', 'a', encoding='utf-8').write('x')\n",
                p.display()
            )
        })
        .unwrap_or_default()
}

async fn write_acp_mock_script_with_optional_log(
    path: &Path,
    start_log: Option<&Path>,
    python_tail: &str,
) {
    let body = format!(
        "#!/usr/bin/env python3\n{}{}",
        acp_mock_start_log_header(start_log),
        python_tail
    );
    tokio::fs::write(path, body.as_bytes()).await.unwrap();
    #[cfg(unix)]
    chmod755_executable(path);
    sync_test_executable(path);
}

/// Writable fake `agent acp` (Unix script with optional startup log hook for spawn races).
pub async fn write_acp_jsonrpc_mock_executable(path: &Path, start_log: Option<&Path>) {
    write_acp_mock_script_with_optional_log(path, start_log, ACP_MOCK_JSONRPC_LOOP_PY).await;
}

/// Successful handshake; `session/prompt` responds with a JSON-RPC error.
pub async fn write_acp_jsonrpc_mock_executable_prompt_fails(path: &Path, start_log: Option<&Path>) {
    write_acp_mock_script_with_optional_log(path, start_log, ACP_MOCK_JSONRPC_PROMPT_FAILS_PY).await;
}

/// Cancel error + slow prompt (see George integration tests).
#[cfg(unix)]
pub async fn write_acp_jsonrpc_mock_cancel_err_slow_prompt(path: &Path) {
    let body = format!("#!/usr/bin/env python3\n{ACP_MOCK_CANCEL_ERR_SLOW_PROMPT_PY}");
    tokio::fs::write(path, body.as_bytes()).await.unwrap();
    chmod755_executable(path);
    sync_test_executable(path);
}

pub fn test_env_lock() -> std::sync::MutexGuard<'static, ()> {
    MALVIN_TEST_ENV_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
