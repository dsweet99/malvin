#![allow(unsafe_code)]

use std::path::Path;

use crate::artifacts::{
    MalvinConfigBackup, backup_workspace_malvin_config_if_present,
    backup_workspace_malvin_config_if_present_with_id, restore_workspace_malvin_config_backup,
    SessionDotfileBackups,
};
use crate::test_utils::with_isolated_home;
use crate::{malvin_config_path, MALVIN_HOME_CONFIG_FILE, seed_malvin_config};

#[test]
fn snapshot_after_ensuring_home_config_records_present_when_file_was_absent() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        assert!(!cfg.exists());
        let bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        assert!(cfg.is_file());
        assert!(matches!(bundle.malvin_config, MalvinConfigBackup::Present(_)));
    });
}

/// Gate loops call `ensure_default_malvin_config_file` during a session. A plain `snapshot` that
/// records `Missing` makes the post-session restore delete that ensured file, so the next
/// iteration snapshots `Missing` again and every restore keeps wiping the config.
#[test]
fn plain_snapshot_missing_restore_wipes_ensure_created_home_config() {
    with_isolated_home(|work| {
        let cfg = malvin_config_path(work);
        assert!(!cfg.exists());
        let bundle = SessionDotfileBackups::snapshot(work).unwrap();
        assert!(matches!(bundle.malvin_config, MalvinConfigBackup::Missing));
        crate::repo_gates::ensure_default_malvin_config_file(work).unwrap();
        assert!(cfg.is_file(), "ensure must materialize home config mid-session");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        assert!(
            !cfg.exists(),
            "Missing restore must delete ensured config — this is the reset bug"
        );
    });
}

#[test]
fn snapshot_after_ensure_breaks_missing_restore_cycle() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "mem_limit_gb = 7\n");
        let cfg = malvin_config_path(work);
        let bundle = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        assert!(matches!(bundle.malvin_config, MalvinConfigBackup::Present(_)));
        seed_malvin_config(work, "TAMPERED\n");
        bundle.restore_excluding_malvin_checks(work).unwrap();
        let restored = std::fs::read_to_string(&cfg).unwrap();
        assert!(
            restored.contains("mem_limit_gb = 7"),
            "restore must put back snapshotted bytes, got: {restored:?}"
        );
        assert!(!restored.contains("TAMPERED"));
    });
}

#[test]
fn malvin_config_backup_skips_when_home_file_missing() {
    with_isolated_home(|work| {
        assert_eq!(
            backup_workspace_malvin_config_if_present(work).unwrap(),
            MalvinConfigBackup::Missing
        );
    });
}

#[test]
fn malvin_config_backup_round_trip_restores_home_file() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_if_present(work).unwrap();
        let MalvinConfigBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };
        assert!(payload.backup_path.is_file());
        seed_malvin_config(work, "MODIFIED\n");
        restore_workspace_malvin_config_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(malvin_config_path(work)).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn malvin_config_backup_missing_restores_by_removing_created_home_file() {
    with_isolated_home(|work| {
        let backup = backup_workspace_malvin_config_if_present(work).unwrap();
        seed_malvin_config(work, "CREATED\n");
        restore_workspace_malvin_config_backup(work, &backup).unwrap();
        assert!(!malvin_config_path(work).exists());
    });
}

#[test]
fn restore_workspace_malvin_config_backup_removes_created_directory_paths() {
    with_isolated_home(|work| {
        let backup = backup_workspace_malvin_config_if_present(work).unwrap();
        let p = malvin_config_path(work);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::create_dir(&p).unwrap();
        restore_workspace_malvin_config_backup(work, &backup).unwrap();
        assert!(!p.exists());
    });
}

#[test]
fn malvin_config_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = Path::new(&home)
            .join(crate::MALVIN_USER_HOME_DIR)
            .join("malvin_config_snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        seed_malvin_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let MalvinConfigBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };

        assert_eq!(
            payload.backup_path.as_path(),
            dir.join("bbbbb").join(MALVIN_HOME_CONFIG_FILE).as_path()
        );
        assert!(payload.backup_path.is_file());
        assert!(!dir.join("aaaaa").join(MALVIN_HOME_CONFIG_FILE).exists());
    });
}
