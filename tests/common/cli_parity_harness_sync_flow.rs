#[cfg(unix)]
pub const CHECK_SYNC_PROMPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_prompts/check_sync.md"
));

#[cfg(unix)]
pub fn acp_mock_sync_header_capture_js() -> String {
    let body = r"const fs = require('fs');
const path = require('path');
const readline = require('readline');
let syncCheckAttempts = 0;
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on('line', (line) => {
  line = line.trim();
  if (!line) {
    return;
  }
  let msg;
  try {
    msg = JSON.parse(line);
  } catch (e) {
    return;
  }
  const mid = msg.method;
  const rid = msg.id;
  if (mid === 'initialize') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'authenticate') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { sessionId: 't1' } }));
  } else if (mid === 'session/prompt') {
    const runRoot = path.join(process.cwd(), '_malvin');
    const runDirs = fs
      .readdirSync(runRoot, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort();
    const markerPath = path.join(runRoot, runDirs[0], 'sync_prompt_headers.txt');
    const promptText = ((((msg.params || {}).prompt || [])[0]) || {}).text || '';
    const hadHeader = promptText.includes('Speak in the first person as malvin');
    const existing = (() => {
      try {
        return fs.readFileSync(markerPath, 'utf8');
      } catch (_) {
        return '';
      }
    })();
    fs.writeFileSync(markerPath, `${existing}${hadHeader ? 'header\n' : 'missing\n'}`, 'utf8');
    if (promptText.includes('KPop: Find a discrepancy between the codebase and')) {
      syncCheckAttempts = syncCheckAttempts + 1;
      if (syncCheckAttempts === 1) {
        fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'needs attention\n', 'utf8');
      } else {
        fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\n', 'utf8');
      }
    } else {
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\n', 'utf8');
    }
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
";
    body.to_string()
}

#[cfg(unix)]
pub fn assert_review_abort_behavior(
    out: &std::process::Output,
    abort_snippet: &str,
    should_stop_prompt: &str,
) {
    assert!(
        !out.status.success(),
        "expected ABORT failure path: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains(abort_snippet),
        "expected review-path ABORT to stop the workflow: {combined:?}"
    );
    assert!(
        !combined.contains(should_stop_prompt),
        "ABORT should stop before Review-2 after Review-1 LGTM: {combined:?}"
    );
}

#[cfg(unix)]
pub fn assert_sync_tamper_flow_restores_grounding_and_fails(
    output: &std::process::Output,
    workspace: &std::path::Path,
) {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "sync should follow expected mock retry-exhaustion path: {combined:?}"
    );
    assert!(
        combined.contains("Did not receive LGTM for check_sync.md within max loops."),
        "sync should fail with expected check_sync exhaustion message: {combined:?}"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.join("grounding.md")).expect("read grounding"),
        "x"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.join(".kissconfig")).expect("read kissconfig"),
        "k\n"
    );
}
