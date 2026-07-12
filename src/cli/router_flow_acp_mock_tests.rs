//! Mock ACP agent helpers for router flow tests.

#[cfg(unix)]
pub(crate) fn install_mock_router_agent_env_with_script(
    workspace: &std::path::Path,
    mock: &std::path::Path,
) -> crate::test_utils::SavedEnvVars {
    #![allow(unsafe_code)]

    let bin_dir = workspace.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    crate::test_agent_client::install_exit_gate_bin(&bin_dir, "kiss", 0);
    let guard = crate::test_utils::SavedEnvVars::capture(&[
        "MALVIN_AGENT_ACP_BIN",
        "PATH",
        "CURSOR_AGENT_API_KEY",
        crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV,
    ]);
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock);
        std::env::set_var("PATH", path);
        std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
    guard
}

#[cfg(unix)]
pub(crate) fn install_mock_router_agent_env(
    workspace: &std::path::Path,
    mock: &std::path::Path,
    continue_after_router_c: bool,
) -> crate::test_utils::SavedEnvVars {
    write_mock_router_agent(mock, continue_after_router_c);
    install_mock_router_agent_env_with_script(workspace, mock)
}

#[cfg(unix)]
pub(crate) fn write_mock_router_agent_session_fail(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = r"#!/usr/bin/env node
const readline = require('readline');
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
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, error: { code: -32603, message: 'session fail' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
";
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub(crate) fn write_mock_router_agent(path: &std::path::Path, continue_after_router_c: bool) {
    use std::os::unix::fs::PermissionsExt;

    let c_text = if continue_after_router_c {
        "CONTINUE_ROUTER\\n"
    } else {
        "router_c done\\n"
    };
    let handler = format!(
        r"    if (!global.pc) global.pc = 0;
    global.pc++;
    const responses = [
      'router_a_1 phase\nCOMPLEXITY_SCORE: 2\n',
      'router_a_2 phase\nCODING_TASK: NO\n',
      'router_b done\n',
      '{c_text}'
    ];
    const text = responses[(global.pc - 1) % responses.length];
    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text }} }} }} }}));"
    );
    let script = format!(
        "#!/usr/bin/env node\n{}\n",
        crate::acp_mock_js("", &handler)
    );
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub(crate) fn write_mock_router_agent_bad_complexity(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let handler = r"    if (!global.pc) global.pc = 0;
    global.pc++;
    const responses = [
      'router_a_1 phase\nCOMPLEXITY_SCORE: not-a-number\n',
      'router_a_2 phase\nmust not reach\n'
    ];
    const text = responses[(global.pc - 1) % responses.length];
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } }));";
    let script = format!(
        "#!/usr/bin/env node\n{}\n",
        crate::acp_mock_js("", handler)
    );
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
#[test]
fn kiss_cov_mock_router_agent_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mock = tmp.path().join("mock");
    write_mock_router_agent(&mock, true);
    assert!(mock.is_file());
    let fail = tmp.path().join("mock-fail");
    write_mock_router_agent_session_fail(&fail);
    assert!(fail.is_file());
    let bad = tmp.path().join("mock-bad");
    write_mock_router_agent_bad_complexity(&bad);
    assert!(bad.is_file());
}
