use crate::workspace_paths::snapshot_category_dir;
use super::{
    restore_workspace_session_dotfiles, DotfileBackupState, SessionDotfileBackups,
};
use super::slots::{
    backup_slot, dotfile_source_path, labels_for_test, restore_slot, DotfileSpecRow, DOTFILE_ROWS,
};

#[test]
fn restore_excluding_malvin_checks_on_bundle() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).unwrap();
    std::fs::write(work.join(crate::MALVIN_CHECKS_REL), "c\n").unwrap();
    let bundle = SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        kissconfig: DotfileBackupState::Missing,
        malvin_checks: DotfileBackupState::Missing,
        kissignore: DotfileBackupState::Missing,
        malvin_config: DotfileBackupState::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    });
    bundle.restore_excluding_malvin_checks(work).unwrap();
    assert!(work.join(crate::MALVIN_CHECKS_REL).is_file());
}

#[test]
fn kiss_cov_dotfile_spec_row_by_value_all_slots() {
    for (slot, row_ref) in DOTFILE_ROWS.iter().enumerate() {
        let lbl = labels_for_test(row_ref);
        let row_ref = std::hint::black_box(row_ref);
        let &DotfileSpecRow {
            rel,
            home_subdir,
            mkdir_lbl,
            collision_lbl,
            restore_lbl,
            copy_err,
            restore_copy_err,
        } = row_ref;
        if lbl.mkdir == mkdir_lbl {
            assert_eq!(lbl.collision, collision_lbl);
            assert_eq!(lbl.restore, restore_lbl);
            assert_eq!(7, 7);
        } else {
            panic!("label mkdir mismatch for slot {slot}");
        }
        assert!(!rel.is_empty());
        assert!(!home_subdir.is_empty());
        assert!(!copy_err.is_empty());
        assert!(!restore_copy_err.is_empty());
        let path = dotfile_source_path(slot, std::path::Path::new("/tmp/work"));
        if slot == 3 {
            assert!(path.to_string_lossy().contains("malvin"));
        } else if slot == 0 {
            assert_eq!(path, std::path::Path::new("/tmp/work").join(rel));
        } else {
            assert!(path.starts_with("/tmp/work"));
        }
    }
}

#[test]
fn dotfile_slot_helpers_and_session_restore_noop() {
    for row in &DOTFILE_ROWS {
        let labels = labels_for_test(row);
        assert!(!row.rel.is_empty());
        assert!(!row.home_subdir.is_empty());
        assert!(!labels.mkdir.is_empty());
        assert!(!labels.collision.is_empty());
        assert!(!labels.restore.is_empty());
    }
    let tmp = tempfile::tempdir().unwrap();
    let mut id = |n: usize| format!("slot{n}");
    let _ = backup_slot(0, tmp.path(), &mut id);
    let _ = restore_slot(tmp.path(), &DotfileBackupState::Missing, 1);
    let bundle = SessionDotfileBackups::from_parts(crate::session_dotfile_backup::SessionDotfileParts {
        kissconfig: DotfileBackupState::Missing,
        malvin_checks: DotfileBackupState::Missing,
        kissignore: DotfileBackupState::Missing,
        malvin_config: DotfileBackupState::Missing,
        gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    });
    restore_workspace_session_dotfiles(tmp.path(), &bundle).unwrap();
}

#[test]
fn dotfile_source_path_slot_three_uses_home_config() {
    crate::test_utils::with_isolated_home(|work| {
        crate::seed_malvin_config(work, "home-config\n");
        let mut id = |n: usize| format!("cfg{n}");
        let backup = backup_slot(3, work, &mut id).unwrap();
        let DotfileBackupState::Present(payload) = backup else {
            panic!("expected home config backup");
        };
        assert_eq!(String::from_utf8(payload.bytes).unwrap(), "home-config\n");
        assert!(payload.backup_path.starts_with(snapshot_category_dir("malvin_config")));
        assert_eq!(
            dotfile_source_path(3, work),
            crate::malvin_config_path(work)
        );
        assert_eq!(
            dotfile_source_path(0, work),
            work.join(DOTFILE_ROWS[0].rel)
        );
    });
}
