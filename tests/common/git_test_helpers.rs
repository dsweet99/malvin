use std::path::Path;

pub fn git_init(project: &Path) {
    assert!(
        std::process::Command::new("git")
            .arg("init")
            .current_dir(project)
            .status()
            .expect("git init")
            .success()
    );
}

pub fn git_commit_all(project: &Path, msg: &str) {
    assert!(
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(project)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .args(["commit", "-m", msg])
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .current_dir(project)
            .status()
            .expect("git commit")
            .success()
    );
}
