use crate::artifacts::{
    GitignoreBackup, backup_workspace_gitignore_if_present,
    backup_workspace_gitignore_if_present_with_id, restore_workspace_gitignore_backup,
};
use crate::test_utils::with_isolated_home;

#[test]
fn gitignore_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        backup_workspace_gitignore_if_present(&work).unwrap(),
        GitignoreBackup::Missing
    );
}

#[test]
fn gitignore_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".gitignore"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_gitignore_if_present(work).unwrap();
        let GitignoreBackup::Present(path) = &backup else {
            panic!("expected backup path");
        };
        assert!(path.is_file());
        std::fs::write(work.join(".gitignore"), "MODIFIED\n").unwrap();
        restore_workspace_gitignore_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".gitignore")).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn gitignore_backup_missing_restores_by_removing_created_workspace_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_gitignore_if_present(&work).unwrap();
    std::fs::write(work.join(".gitignore"), "CREATED\n").unwrap();
    restore_workspace_gitignore_backup(&work, &backup).unwrap();
    assert!(!work.join(".gitignore").exists());
}

#[test]
fn gitignore_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = std::path::Path::new(&home)
            .join(".malvin")
            .join("gitignore_snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        std::fs::write(work.join(".gitignore"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_gitignore_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let GitignoreBackup::Present(path) = &backup else {
            panic!("expected backup path");
        };
        assert_eq!(path.parent(), Some(dir.join("bbbbb").as_path()));
        assert!(dir.join("bbbbb").join(".gitignore").is_file());
    });
}
