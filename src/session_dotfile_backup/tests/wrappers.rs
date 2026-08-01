//! Behavioral coverage for thin wrapper entrypoints in [`super::super::wrappers`].

use crate::artifacts::{
    backup_workspace_malvin_config_if_present, backup_workspace_malvin_config_if_present_with_id,
    restore_workspace_malvin_config_backup, MalvinConfigBackup,
};
use crate::test_utils::with_isolated_home;
use crate::{malvin_config_path, seed_malvin_config};

#[test]
fn wrapper_malvin_config_backup_and_restore_round_trip() {
    with_isolated_home(|work| {
        seed_malvin_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_if_present(work).unwrap();
        let MalvinConfigBackup::Present(payload) = &backup else {
            panic!("expected malvin_config backup");
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
fn wrapper_malvin_config_backup_with_id_retries_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = std::path::Path::new(&home)
            .join(".malvin")
            .join("snapshots")
            .join("malvin_config");
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
            panic!("expected backup");
        };
        assert!(payload.backup_path.starts_with(dir.join("bbbbb")));
    });
}
