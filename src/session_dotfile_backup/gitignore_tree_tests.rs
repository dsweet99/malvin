use super::{
    backup_workspace_gitignore_if_present, collect_workspace_gitignore_relpaths,
    restore_workspace_gitignore_backup, GitignoreBackup,
};
use crate::test_utils::with_isolated_home;
use std::path::{Path, PathBuf};

fn init_git_repo(work: &Path) {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(work)
        .status()
        .expect("git init");
}

#[test]
fn collect_finds_root_and_nested_gitignore_files() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(work.join("pkg/.cache")).unwrap();
    std::fs::write(work.join(".gitignore"), "root\n").unwrap();
    std::fs::write(work.join("pkg/.gitignore"), "pkg\n").unwrap();
    std::fs::write(work.join("pkg/.cache/.gitignore"), "cache\n").unwrap();
    init_git_repo(&work);

    let rels = collect_workspace_gitignore_relpaths(&work);
    assert_eq!(
        rels,
        vec![
            PathBuf::from(".gitignore"),
            PathBuf::from("pkg/.cache/.gitignore"),
            PathBuf::from("pkg/.gitignore"),
        ]
    );
}

#[test]
fn collect_non_git_workspace_only_checks_root_gitignore_not_subdirs() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work.join("nested/deep")).unwrap();
    std::fs::write(work.join("nested/.gitignore"), "nested\n").unwrap();
    std::fs::write(work.join("nested/deep/.gitignore"), "deep\n").unwrap();
    assert!(collect_workspace_gitignore_relpaths(work).is_empty());

    std::fs::write(work.join(".gitignore"), "root\n").unwrap();
    assert_eq!(
        collect_workspace_gitignore_relpaths(work),
        vec![PathBuf::from(".gitignore")]
    );
}

fn seed_nested_gitignore_repo(work: &Path) {
    std::fs::create_dir_all(work.join("pkg")).unwrap();
    std::fs::write(work.join(".gitignore"), "root\n").unwrap();
    std::fs::write(work.join("pkg/.gitignore"), "pkg\n").unwrap();
    init_git_repo(work);
}

fn tamper_gitignore_tree(work: &Path) {
    std::fs::write(work.join(".gitignore"), "tampered-root\n").unwrap();
    std::fs::write(work.join("pkg/.gitignore"), "tampered-pkg\n").unwrap();
    std::fs::create_dir_all(work.join("new")).unwrap();
    std::fs::write(work.join("new/.gitignore"), "agent-created\n").unwrap();
}

#[test]
fn nested_gitignore_round_trip_restores_tree_and_removes_agent_created_files() {
    with_isolated_home(|work| {
        seed_nested_gitignore_repo(work);
        let backup =
            super::backup_workspace_gitignore_if_present_with_id(work, &mut |n| format!("gi{n}"))
                .unwrap();
        let GitignoreBackup::Present { backup_root, files } = &backup else {
            panic!("expected gitignore tree backup");
        };
        assert!(backup_root.starts_with(
            crate::workspace_paths::snapshot_category_dir("gitignore")
        ));
        assert_eq!(files.len(), 2);

        tamper_gitignore_tree(work);
        restore_workspace_gitignore_backup(work, &backup).unwrap();
        assert_gitignore_contents(work, ".gitignore", "root\n");
        assert_gitignore_contents(work, "pkg/.gitignore", "pkg\n");
        assert!(!work.join("new/.gitignore").exists());
    });
}

fn assert_gitignore_contents(work: &Path, rel: &str, expected: &str) {
    assert_eq!(
        std::fs::read_to_string(work.join(rel)).unwrap(),
        expected
    );
}

#[test]
fn poisoned_disk_snapshot_does_not_change_restored_gitignore_content() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".gitignore"), "ORIGINAL\n").unwrap();
        let backup =
            super::backup_workspace_gitignore_if_present_with_id(work, &mut |n| format!("poison{n}"))
                .unwrap();
        let GitignoreBackup::Present { backup_root, .. } = &backup else {
            panic!("expected backup");
        };
        std::fs::write(backup_root.join(".gitignore"), "POISONED\n").unwrap();
        std::fs::write(work.join(".gitignore"), "AGENT\n").unwrap();

        restore_workspace_gitignore_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".gitignore")).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn backup_workspace_gitignore_if_present_delegates_to_tree_backup() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".gitignore"), "root\n").unwrap();
        let backup = backup_workspace_gitignore_if_present(work).expect("backup");
        assert!(matches!(backup, GitignoreBackup::Present { .. }));
    });
}
