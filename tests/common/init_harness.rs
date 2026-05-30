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

pub use super::init_harness_run::{
    malvin_init_output, malvin_init_output_in_place, malvin_init_output_with_home,
};

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
    home: tempfile::TempDir,
}

impl InitOk {
    pub fn new(init_args: &[&str]) -> Self {
        let project = tempfile::tempdir().unwrap();
        let (out, home) = malvin_init_output(project.path(), init_args);
        assert!(out.status.success(), "malvin init failed: {out:?}");
        Self { project, home }
    }

    #[must_use]
    pub fn home_path(&self) -> &Path {
        self.home.path()
    }

    pub fn path(&self) -> &Path {
        self.project.path()
    }

    pub fn read_rel(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path().join(rel)).unwrap()
    }
}
