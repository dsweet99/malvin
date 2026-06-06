use super::alloc::malvin_home_dir;
use super::{
    restore_workspace_session_dotfiles, DotfileBackupState, SessionDotfileBackups,
};
use super::slots::{backup_slot, dotfile_source_path, labels_for_test, restore_slot, DOTFILE_ROWS};

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
        gitignore: DotfileBackupState::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    });
    bundle.restore_excluding_malvin_checks(work).unwrap();
    assert!(work.join(crate::MALVIN_CHECKS_REL).is_file());
}

#[test]
fn dotfile_slot_helpers_and_session_restore_noop() {
    for row in &DOTFILE_ROWS {
        let _ = labels_for_test(row);
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
        gitignore: DotfileBackupState::Missing,
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
        assert!(payload.backup_path.starts_with(
            malvin_home_dir().join(".malvin").join("malvin_config_snapshots")
        ));
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
