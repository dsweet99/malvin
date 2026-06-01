use std::path::Path;
use std::process::Command;

use super::write_mock_executable;

pub fn acp_mock_init_js() -> String {
    let body = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const isKpop = promptText.includes('KPOP') || promptText.includes('init_constraints');
    if (isKpop) {
      const targetMatch = promptText.match(/exp_log_[^\s`]+\.md/);
      const target = targetMatch ? targetMatch[0] : null;
      const os = require('os');
      const root = path.join(os.homedir(), '.malvin', 'logs');
      if (fs.existsSync(root)) {
        outer: for (const hash of fs.readdirSync(root, { withFileTypes: true }).filter((e) => e.isDirectory())) {
          const bucket = path.join(root, hash.name);
          const runs = fs.readdirSync(bucket, { withFileTypes: true })
            .filter((e) => e.isDirectory()).map((e) => e.name).sort().reverse();
          for (const run of runs) {
            const kpopDir = path.join(bucket, run, '_kpop');
            if (!fs.existsSync(kpopDir)) continue;
            const names = target
              ? [target]
              : fs.readdirSync(kpopDir).filter((n) => /_g\d+\.md$/.test(n)).sort();
            for (const name of names) {
              if (!name.startsWith('exp_log_') || !name.endsWith('.md')) continue;
              fs.appendFileSync(path.join(kpopDir, name), '\n## KPOP_SOLVED\n');
              break outer;
            }
          }
        }
      }
      const checksPath = path.join(process.cwd(), '.malvin', 'checks');
      fs.mkdirSync(path.dirname(checksPath), { recursive: true });
      fs.writeFileSync(checksPath, 'kiss check\n');
    }";
    let kpop_done =
        super::acp_core::session_update_chunk_line("agent_message_chunk", r"'init kpop ok\n'");
    super::acp_core::acp_mock_js("", &format!("{body}\n    if (isKpop) {{ {kpop_done} }}"))
}

fn spawn_malvin_init(
    project: &Path,
    home: &Path,
    init_args: &[&str],
    in_place: bool,
) -> std::process::Output {
    let pre_commit_home = tempfile::tempdir().expect("pre-commit home tempdir");
    let mock_bin = home.join("mock-acp-init");
    write_mock_executable(&mock_bin, &acp_mock_init_js());
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.arg("init");
    for a in init_args {
        cmd.arg(a);
    }
    if in_place {
        cmd.current_dir(project);
    } else {
        cmd.args(["--path"]).arg(project);
    }
    cmd.env("HOME", home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", mock_bin.as_os_str())
        .env("PRE_COMMIT_HOME", pre_commit_home.path());
    cmd.output().expect("spawn malvin init")
}

/// Run `malvin init` with CWD set to `project` (no `--path`), matching in-place user usage.
pub fn malvin_init_output_in_place(
    project: &Path,
    init_args: &[&str],
) -> (std::process::Output, tempfile::TempDir) {
    let mock_home = tempfile::tempdir().expect("mock home tempdir");
    let out = spawn_malvin_init(project, mock_home.path(), init_args, true);
    (out, mock_home)
}

pub fn malvin_init_output(
    project: &Path,
    init_args: &[&str],
) -> (std::process::Output, tempfile::TempDir) {
    let mock_home = tempfile::tempdir().expect("mock home tempdir");
    let out = spawn_malvin_init(project, mock_home.path(), init_args, false);
    (out, mock_home)
}

pub fn malvin_init_output_with_home(
    project: &Path,
    home: &Path,
    init_args: &[&str],
) -> std::process::Output {
    spawn_malvin_init(project, home, init_args, false)
}
