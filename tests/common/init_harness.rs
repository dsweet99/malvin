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

pub use super::init_harness_run::{
    malvin_init_output, malvin_init_output_in_place, malvin_init_output_with_home,
};

pub fn run_has_mpc_plan_done(run_dir: &Path) -> bool {
    let path = run_dir.join("_kpop").join("mpc_plan.md");
    std::fs::read_to_string(path)
        .is_ok_and(|text| text.trim() == "DONE")
}

pub fn gate_exp_logs_with_kpop_solved(run_dir: &Path) -> Vec<std::path::PathBuf> {
    if run_has_mpc_plan_done(run_dir) {
        vec![run_dir.join("_kpop").join("mpc_plan.md")]
    } else {
        Vec::new()
    }
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
