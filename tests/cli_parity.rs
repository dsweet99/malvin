use std::path::Path;
use std::process::Command;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const ROOT_GITIGNORE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/.gitignore"));
const INIT_TEMPLATE_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));
#[cfg(unix)]
const CODE_STREAMING_MOCK: &str = r"const readline = require('readline');
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
  } else if (mid === 'session/prompt') {
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      method: 'session/update',
      params: {
        update: {
          sessionUpdate: 'agent_message_chunk',
          content: { type: 'text', text: 'agent message\n' }
        }
      }
    }));
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});";

fn check_ignored(repo: &Path, rel_path: &str) -> bool {
    Command::new("git")
        .current_dir(repo)
        .args(["check-ignore", "-q", rel_path])
        .status()
        .unwrap_or_else(|e| panic!("git check-ignore spawn failed: {e}"))
        .success()
}

#[cfg(unix)]
fn write_mock_executable(path: &Path) {
    let script = format!("#!/usr/bin/env node\n{CODE_STREAMING_MOCK}");
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
fn write_fake_kiss(path: &Path) {
    std::fs::write(path, "#!/usr/bin/env sh\nexit 0\n").expect("write kiss");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
fn run_code_max_loops_zero_with_mock_opts(no_tee: bool) -> std::process::Output {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    let mock = root.path().join("mock-agent-acp-code");
    write_mock_executable(&mock);
    let kiss = bin_dir.join("kiss");
    write_fake_kiss(&kiss);
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());
    let mut args = vec!["code", "--trust-the-plan", "--no-learn", "--max-loops", "0", "ship it"];
    if no_tee {
        args.insert(0, "--no-tee");
    }
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("PATH", path)
        .args(args)
        .output()
        .expect("spawn malvin code")
}

#[cfg(unix)]
fn run_code_max_loops_zero_with_mock() -> std::process::Output {
    run_code_max_loops_zero_with_mock_opts(true)
}

#[cfg(unix)]
fn run_code_max_loops_zero_with_mock_stdout() -> std::process::Output {
    run_code_max_loops_zero_with_mock_opts(false)
}

#[test]
#[cfg(unix)]
fn max_loops_zero_skips_review_attempts_and_fails() {
    let out = run_code_max_loops_zero_with_mock();
    assert!(!out.status.success(), "malvin code unexpectedly succeeded: {out:?}");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("Did not receive LGTM for review_1.md within max loops."),
        "missing max-loops failure message: {combined:?}"
    );
    assert_eq!(
        combined.matches("Implement").count(),
        1,
        "expected one implement phase: {combined:?}"
    );
    assert!(
        !combined.contains("Review-1 (attempt 1)"),
        "review attempt must not run when --max-loops=0: {combined:?}"
    );
}

#[test]
#[cfg(unix)]
fn code_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_code_max_loops_zero_with_mock_stdout();
    assert!(!out.status.success(), "expected max-loops failure path: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("agent message"),
        "expected parsed agent output on stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
}


#[test]
fn root_gitignore_ignores_malvin_logs_and_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        check_ignored(root, "_malvin/dummy_stamp/plan.md"),
        "expected _malvin/ run dirs to be ignored"
    );
    assert!(
        check_ignored(root, "log"),
        "expected root log file to be ignored"
    );
    assert!(
        check_ignored(root, "log_2"),
        "expected root log_2 to be ignored"
    );
    assert!(
        check_ignored(root, "target/debug/malvin"),
        "expected Rust target/ tree to be ignored"
    );
    assert!(
        !check_ignored(root, "README.md"),
        "expected README.md not to be ignored"
    );
}

#[test]
fn init_template_gitignore_is_consistent_with_git_check_ignore() {
    const TEMPLATE: &str = INIT_TEMPLATE_GITIGNORE;
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join(".gitignore"), TEMPLATE).unwrap();
    let st = Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    assert!(st.success(), "git init failed");
    assert!(
        check_ignored(tmp.path(), "_malvin/x/plan.md"),
        "template should ignore _malvin/ runs"
    );
    assert!(
        check_ignored(tmp.path(), "log"),
        "template should ignore root log"
    );
    assert!(
        check_ignored(tmp.path(), "log_2"),
        "template should ignore root log_2"
    );
    assert!(
        check_ignored(tmp.path(), "target/release/foo"),
        "template should ignore Rust target/"
    );
    assert!(
        !check_ignored(tmp.path(), "src/lib.rs"),
        "template should not ignore normal sources"
    );
    assert!(
        check_ignored(tmp.path(), "pkg/__pycache__/x.py"),
        "template should ignore sources under nested __pycache__ dirs (not only *.pyc)"
    );
    assert!(
        check_ignored(tmp.path(), "lib/foo.pyc"),
        "template should ignore .pyc via **/*.py[cod]"
    );
}

#[test]
fn init_template_gitignore_matches_root_python_ignore_patterns() {
    for line in ["**/__pycache__/", "**/*.py[cod]"] {
        assert!(
            ROOT_GITIGNORE.lines().any(|l| l.trim() == line),
            "repo root .gitignore must list {line:?}"
        );
        assert!(
            INIT_TEMPLATE_GITIGNORE.lines().any(|l| l.trim() == line),
            "malvin init template .gitignore must list {line:?} so new repos match Malvin's own ignores"
        );
    }
}

#[test]
fn kpop_p_creative_runtime_gate_contract() {
    assert!(!malvin::kpop_creative_enabled(0.0));
    assert!(!malvin::kpop_creative_enabled(-0.1));
    assert!(!malvin::kpop_creative_enabled(f64::INFINITY));
    assert!(!malvin::kpop_creative_enabled(f64::NAN));
    assert!(malvin::kpop_creative_enabled(0.1));
}


