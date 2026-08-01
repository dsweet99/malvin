use crate::session_dotfile_backup::tree_test_support::init_git_repo;
use super::{
    backup_workspace_vision_if_present, collect_workspace_vision_relpaths,
    restore_workspace_vision_backup, VisionBackup,
};
use crate::test_utils::with_isolated_home;
use std::path::{Path, PathBuf};

#[test]
fn collect_finds_root_and_nested_vision_files() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(work.join("pkg/.cache")).unwrap();
    std::fs::write(work.join("VISION.md"), "root\n").unwrap();
    std::fs::write(work.join("pkg/VISION.md"), "pkg\n").unwrap();
    std::fs::write(work.join("pkg/.cache/VISION.md"), "cache\n").unwrap();
    init_git_repo(&work);

    let rels = collect_workspace_vision_relpaths(&work);
    assert_eq!(
        rels,
        vec![
            PathBuf::from("VISION.md"),
            PathBuf::from("pkg/.cache/VISION.md"),
            PathBuf::from("pkg/VISION.md"),
        ]
    );
}

#[test]
fn collect_non_git_workspace_only_checks_root_vision_not_subdirs() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work.join("nested/deep")).unwrap();
    std::fs::write(work.join("nested/VISION.md"), "nested\n").unwrap();
    std::fs::write(work.join("nested/deep/VISION.md"), "deep\n").unwrap();
    assert!(collect_workspace_vision_relpaths(work).is_empty());

    std::fs::write(work.join("VISION.md"), "root\n").unwrap();
    assert_eq!(
        collect_workspace_vision_relpaths(work),
        vec![PathBuf::from("VISION.md")]
    );
}

pub(crate) fn seed_nested_vision_repo(work: &Path) {
    std::fs::create_dir_all(work.join("pkg")).unwrap();
    std::fs::write(work.join("VISION.md"), "root\n").unwrap();
    std::fs::write(work.join("pkg/VISION.md"), "pkg\n").unwrap();
    init_git_repo(work);
}

pub(crate) fn tamper_vision_tree(work: &Path) {
    std::fs::write(work.join("VISION.md"), "tampered-root\n").unwrap();
    std::fs::write(work.join("pkg/VISION.md"), "tampered-pkg\n").unwrap();
    std::fs::create_dir_all(work.join("new")).unwrap();
    std::fs::write(work.join("new/VISION.md"), "agent-created\n").unwrap();
}

#[test]
fn nested_vision_round_trip_restores_tree_and_removes_agent_created_files() {
    with_isolated_home(|work| {
        seed_nested_vision_repo(work);
        let backup =
            super::backup_workspace_vision_if_present_with_id(work, &mut |n| format!("vi{n}"))
                .unwrap();
        let VisionBackup::Present { backup_root, files } = &backup else {
            panic!("expected vision tree backup");
        };
        assert!(backup_root.starts_with(
            crate::workspace_paths::snapshot_category_dir("vision")
        ));
        assert_eq!(files.len(), 2);

        tamper_vision_tree(work);
        restore_workspace_vision_backup(work, &backup).unwrap();
        assert_vision_contents(work, "VISION.md", "root\n");
        assert_vision_contents(work, "pkg/VISION.md", "pkg\n");
        assert!(!work.join("new/VISION.md").exists());
    });
}

pub(crate) fn assert_vision_contents(work: &Path, rel: &str, expected: &str) {
    assert_eq!(
        std::fs::read_to_string(work.join(rel)).unwrap(),
        expected
    );
}

#[test]
fn poisoned_disk_snapshot_does_not_change_restored_vision_content() {
    with_isolated_home(|work| {
        std::fs::write(work.join("VISION.md"), "ORIGINAL\n").unwrap();
        let backup =
            super::backup_workspace_vision_if_present_with_id(work, &mut |n| format!("poison{n}"))
                .unwrap();
        let VisionBackup::Present { backup_root, .. } = &backup else {
            panic!("expected backup");
        };
        std::fs::write(backup_root.join("VISION.md"), "POISONED\n").unwrap();
        std::fs::write(work.join("VISION.md"), "AGENT\n").unwrap();

        restore_workspace_vision_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("VISION.md")).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn backup_workspace_vision_if_present_delegates_to_tree_backup() {
    with_isolated_home(|work| {
        std::fs::write(work.join("VISION.md"), "root\n").unwrap();
        let backup = backup_workspace_vision_if_present(work).expect("backup");
        assert!(matches!(backup, VisionBackup::Present { .. }));
    });
}
