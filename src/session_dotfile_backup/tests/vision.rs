use crate::artifacts::VisionBackup;
use crate::test_support::test_utils::with_isolated_home;
use crate::workspace::workspace_paths::snapshot_category_dir;

#[test]
fn vision_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        VisionBackup::backup_if_present(&work).unwrap(),
        VisionBackup::Missing
    );
}

#[test]
fn vision_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::write(work.join("VISION.md"), "ORIGINAL\n").unwrap();
        let backup = VisionBackup::backup_if_present(work).unwrap();
        let VisionBackup::Present { backup_root, files } = &backup else {
            panic!("expected backup path");
        };
        assert!(backup_root.join("VISION.md").is_file());
        assert_eq!(files.len(), 1);
        std::fs::write(work.join("VISION.md"), "MODIFIED\n").unwrap();
        backup.restore(work).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join("VISION.md")).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn vision_backup_missing_restores_by_removing_created_workspace_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = VisionBackup::backup_if_present(&work).unwrap();
    std::fs::write(work.join("VISION.md"), "CREATED\n").unwrap();
    backup.restore(&work).unwrap();
    assert!(!work.join("VISION.md").exists());
}

#[test]
fn vision_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let dir = snapshot_category_dir("vision");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        std::fs::write(work.join("VISION.md"), "ORIGINAL\n").unwrap();
        let backup = VisionBackup::backup_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let VisionBackup::Present { backup_root, .. } = &backup else {
            panic!("expected backup path");
        };
        assert_eq!(backup_root.as_path(), dir.join("bbbbb").as_path());
        assert!(dir.join("bbbbb").join("VISION.md").is_file());
    });
}
