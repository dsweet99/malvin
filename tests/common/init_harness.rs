use std::path::Path;
use std::process::Command;

pub fn git_init(project: &Path) {
    assert!(
        Command::new("git")
            .arg("init")
            .current_dir(project)
            .status()
            .expect("git init")
            .success()
    );
}

pub fn assert_deduped_precommit_checks(checks: &str) {
    let lines: Vec<&str> = checks
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert!(
        lines.first() == Some(&"kiss check"),
        "kiss check must be first; got: {lines:?}"
    );
    assert_eq!(
        lines.iter().filter(|l| **l == "ruff check .").count(),
        1,
        "expected exactly one deduped ruff line; got: {lines:?}"
    );
    assert!(
        lines.contains(&"python3 -m compileall -q ."),
        "expected pre-commit compileall hook; got: {lines:?}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("compileall -q src")),
        "Makefile lint must not override pre-commit signal; got: {lines:?}"
    );
    assert_eq!(
        lines.len(),
        3,
        "expected kiss + deduped ruff + compileall; got: {lines:?}"
    );
}

pub fn seed_precommit_dedupe_fixture(project: &Path) {
    git_init(project);
    std::fs::write(project.join("lib.py"), "x = 1\n").expect("write lib.py");
    std::fs::write(
        project.join(".pre-commit-config.yaml"),
        "repos:\n- repo: local\n  hooks:\n  - id: ruff-a\n    entry: ruff check .\n    language: system\n  - id: ruff-b\n    entry: ruff check .\n    language: system\n  - id: compile\n    entry: python3 -m compileall -q .\n    language: system\n",
    )
    .expect("write pre-commit config");
    std::fs::write(
        project.join("Makefile"),
        "lint:\n\tpython3 -m compileall -q src\n",
    )
    .expect("write makefile");
    git_commit_all(project, "seed tooling");
}

pub fn acp_mock_init_js() -> String {
    let body = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const isKpop = promptText.includes('KPOP') || promptText.includes('init_constraints');
    if (isKpop) {
      const targetMatch = promptText.match(/exp_log_[^\s`]+\.md/);
      const target = targetMatch ? targetMatch[0] : null;
      const root = path.join(process.cwd(), '.malvin', 'logs');
      if (fs.existsSync(root)) {
        const runs = fs.readdirSync(root, { withFileTypes: true })
          .filter((e) => e.isDirectory())
          .map((e) => e.name)
          .sort()
          .reverse();
        outer: for (const run of runs) {
          const kpopDir = path.join(root, run, '_kpop');
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
      const checksPath = path.join(process.cwd(), '.malvin', 'checks');
      fs.mkdirSync(path.dirname(checksPath), { recursive: true });
      fs.writeFileSync(checksPath, 'kiss check\n');
    }";
    let kpop_done =
        super::acp_core::session_update_chunk_line("agent_message_chunk", r"'init kpop ok\n'");
    let summary_done =
        super::acp_core::session_update_chunk_line("agent_message_chunk", r"'init summary ok\n'");
    super::acp_core::acp_mock_js(
        "",
        &format!(
            "{body}\n    if (!isKpop) {{ {summary_done} }} else {{ {kpop_done} }}"
        ),
    )
}

pub fn malvin_init_output(project: &Path, init_args: &[&str]) -> std::process::Output {
    let mock_home = tempfile::tempdir().expect("mock home tempdir");
    let pre_commit_home = tempfile::tempdir().expect("pre-commit home tempdir");
    let mock_bin = mock_home.path().join("mock-acp-init");
    let js = acp_mock_init_js();
    super::write_mock_executable(&mock_bin, &js);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.arg("init");
    for a in init_args {
        cmd.arg(a);
    }
    cmd.args(["--path"]).arg(project);
    cmd.env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", mock_bin.as_os_str())
        .env("PRE_COMMIT_HOME", pre_commit_home.path());
    cmd.output().expect("spawn malvin init")
}

pub fn gate_exp_logs_with_kpop_solved(run_dir: &Path) -> Vec<std::path::PathBuf> {
    super::gate_exp_logs_in_run(run_dir)
        .into_iter()
        .filter(|p| {
            std::fs::read_to_string(p).is_ok_and(|text| text.contains("## KPOP_SOLVED"))
        })
        .collect()
}

pub fn git_stdout(project: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(project)
        .output()
        .expect("git");
    assert!(out.status.success(), "git failed: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn git_show_rev_path(project: &Path, rev_path: &str) -> String {
    let out = Command::new("git")
        .args(["show", rev_path])
        .current_dir(project)
        .output()
        .expect("git show");
    assert!(out.status.success(), "git show: {out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

pub fn tempdir_seeded_dirty_keep() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    git_init(project.path());
    let keep = project.path().join("keep.txt");
    std::fs::write(&keep, "before\n").expect("write keep");
    git_commit_all(project.path(), "seed repo");
    std::fs::write(&keep, "after\n").expect("dirty tracked file");
    project
}

pub fn assert_git_branch_main(project: &Path) {
    assert_eq!(
        git_stdout(project, &["branch", "--show-current"]).trim(),
        "main"
    );
}

pub fn assert_git_head_commit_count(project: &Path, expected: &str) {
    assert_eq!(
        git_stdout(project, &["rev-list", "--count", "HEAD"]).trim(),
        expected
    );
}

pub fn git_commit_all(project: &Path, msg: &str) {
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.com",
                "add",
                ".",
            ])
            .current_dir(project)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-c",
                "user.name=test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                msg,
            ])
            .current_dir(project)
            .status()
            .expect("git commit")
            .success()
    );
}

pub struct InitOk {
    pub project: tempfile::TempDir,
}

impl InitOk {
    pub fn new(init_args: &[&str]) -> Self {
        let project = tempfile::tempdir().unwrap();
        let out = malvin_init_output(project.path(), init_args);
        assert!(out.status.success(), "malvin init failed: {out:?}");
        Self { project }
    }

    pub fn path(&self) -> &Path {
        self.project.path()
    }

    pub fn read_rel(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path().join(rel)).unwrap()
    }
}
