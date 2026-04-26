//! Integration tests for `malvin init`.

use std::process::Command;

#[test]
fn malvin_init_fails_fast_when_pre_commit_missing_from_path() {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    let project = tempfile::tempdir().unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .args(["init", "python", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");

    assert!(
        !out.status.success(),
        "expected non-zero exit; stdout/stderr: {out:?}"
    );
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        msg.contains("pre-commit"),
        "expected explicit pre-commit hint; got: {msg:?}"
    );
}

#[test]
fn malvin_init_rejects_unknown_language() {
    let project = tempfile::tempdir().unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "javascript", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");

    assert!(!out.status.success(), "should reject unknown language");
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        msg.contains("Unknown language"),
        "should mention unknown language: {msg:?}"
    );
}

#[test]
fn malvin_init_rejects_no_languages() {
    let project = tempfile::tempdir().unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");

    assert!(!out.status.success(), "should reject missing language args");
}

#[test]
fn malvin_init_creates_expected_files_for_python_only() {
    let project = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(project.path())
        .output()
        .expect("git init");

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "python", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");
    assert!(out.status.success(), "malvin init failed: {out:?}");

    let pre_commit =
        std::fs::read_to_string(project.path().join(".pre-commit-config.yaml")).unwrap();
    assert!(
        pre_commit.contains("ruff"),
        "python-only should have ruff hook"
    );
    assert!(
        !pre_commit.contains("clippy"),
        "python-only should not have clippy hook"
    );
    assert!(pre_commit.contains("kiss"), "should always have kiss hook");
    assert!(
        pre_commit.contains("check-untracked"),
        "should always have check-untracked hook"
    );

    let grounding = std::fs::read_to_string(project.path().join("grounding.md")).unwrap();
    assert!(
        grounding.contains("in Python"),
        "grounding should mention Python"
    );
    assert!(
        !grounding.contains("{{languages}}"),
        "grounding should not have unreplaced placeholder"
    );

    assert!(project.path().join(".gitignore").exists());
    assert!(project.path().join(".kissignore").exists());
    assert!(project.path().join("admin/check_untracked.sh").exists());
}

#[test]
fn malvin_init_creates_expected_files_for_rust_only() {
    let project = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(project.path())
        .output()
        .expect("git init");

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "rust", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");
    assert!(out.status.success(), "malvin init failed: {out:?}");

    let pre_commit =
        std::fs::read_to_string(project.path().join(".pre-commit-config.yaml")).unwrap();
    assert!(
        !pre_commit.contains("ruff"),
        "rust-only should not have ruff hook"
    );
    assert!(
        pre_commit.contains("clippy"),
        "rust-only should have clippy hook"
    );

    let grounding = std::fs::read_to_string(project.path().join("grounding.md")).unwrap();
    assert!(
        grounding.contains("in Rust"),
        "grounding should mention Rust"
    );
}

#[test]
fn malvin_init_creates_expected_files_for_both_languages() {
    let project = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(project.path())
        .output()
        .expect("git init");

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "python", "rust", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");
    assert!(out.status.success(), "malvin init failed: {out:?}");

    let pre_commit =
        std::fs::read_to_string(project.path().join(".pre-commit-config.yaml")).unwrap();
    assert!(
        pre_commit.contains("ruff"),
        "both languages should have ruff hook"
    );
    assert!(
        pre_commit.contains("clippy"),
        "both languages should have clippy hook"
    );

    let grounding = std::fs::read_to_string(project.path().join("grounding.md")).unwrap();
    assert!(
        grounding.contains("in Python and Rust"),
        "grounding should mention both languages"
    );
}

#[test]
fn malvin_init_language_args_are_case_insensitive() {
    let project = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(project.path())
        .output()
        .expect("git init");

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "PYTHON", "Rust", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");
    assert!(
        out.status.success(),
        "malvin init with mixed case should succeed: {out:?}"
    );

    let grounding = std::fs::read_to_string(project.path().join("grounding.md")).unwrap();
    assert!(
        grounding.contains("in Python and Rust"),
        "grounding should have proper casing"
    );
}

#[test]
fn malvin_init_creates_initial_commit_on_main_and_installs_llm_style_for_fresh_repo() {
    let project = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(project.path())
        .output()
        .expect("git init");

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "python", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");
    assert!(out.status.success(), "malvin init failed: {out:?}");

    assert!(
        project.path().join(".llm_style/style.md").exists(),
        "init should install .llm_style/style.md"
    );

    let head = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project.path())
        .output()
        .expect("git branch --show-current");
    assert!(
        head.status.success(),
        "git branch --show-current failed: {head:?}"
    );
    assert_eq!(
        String::from_utf8_lossy(&head.stdout).trim(),
        "main",
        "init should leave HEAD on main"
    );

    let commit_count = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(project.path())
        .output()
        .expect("git rev-list --count HEAD");
    assert!(
        commit_count.status.success(),
        "git rev-list --count HEAD failed: {commit_count:?}"
    );
    assert_eq!(
        String::from_utf8_lossy(&commit_count.stdout).trim(),
        "1",
        "init should create exactly one initial commit in a fresh repo"
    );

    let tracked = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .current_dir(project.path())
        .output()
        .expect("git ls-tree");
    assert!(tracked.status.success(), "git ls-tree failed: {tracked:?}");
    let tracked_stdout = String::from_utf8_lossy(&tracked.stdout);
    assert!(
        tracked_stdout.contains("grounding.md"),
        "initial commit should include grounding.md"
    );
    assert!(
        tracked_stdout.contains(".llm_style/style.md"),
        "initial commit should include .llm_style/style.md"
    );
}

#[test]
fn malvin_init_does_not_autocommit_preexisting_repo_changes() {
    let project = tempfile::tempdir().unwrap();
    Command::new("git")
        .arg("init")
        .current_dir(project.path())
        .output()
        .expect("git init");

    let keep = project.path().join("keep.txt");
    std::fs::write(&keep, "before\n").expect("write keep");
    let initial_commit = Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.com",
            "add",
            ".",
        ])
        .current_dir(project.path())
        .output()
        .expect("git add");
    assert!(
        initial_commit.status.success(),
        "git add failed: {initial_commit:?}"
    );
    let initial_commit = Command::new("git")
        .args([
            "-c",
            "user.name=test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "seed repo",
        ])
        .current_dir(project.path())
        .output()
        .expect("git commit");
    assert!(
        initial_commit.status.success(),
        "seed commit failed: {initial_commit:?}"
    );

    std::fs::write(&keep, "after\n").expect("dirty tracked file");

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["init", "python", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");
    assert!(out.status.success(), "malvin init failed: {out:?}");

    let commit_count = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(project.path())
        .output()
        .expect("git rev-list --count HEAD");
    assert!(
        commit_count.status.success(),
        "git rev-list --count HEAD failed: {commit_count:?}"
    );
    assert_eq!(
        String::from_utf8_lossy(&commit_count.stdout).trim(),
        "1",
        "init should not create a new commit when bootstrapping an existing repo"
    );

    let tracked = Command::new("git")
        .args(["show", "HEAD:keep.txt"])
        .current_dir(project.path())
        .output()
        .expect("git show HEAD:keep.txt");
    assert!(tracked.status.success(), "git show failed: {tracked:?}");
    assert_eq!(
        String::from_utf8_lossy(&tracked.stdout),
        "before\n",
        "existing tracked content should not be silently rewritten into a new init commit"
    );
}
