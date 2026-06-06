use crate::artifacts::{
    MalvinConfigWorkspaceBackup, backup_workspace_malvin_config_workspace_if_present,
    restore_workspace_malvin_config_workspace_backup,
};
use crate::test_utils::with_isolated_home;
use crate::MALVIN_CONFIG_REL;

#[test]
fn malvin_config_workspace_backup_skips_when_file_missing() {
    with_isolated_home(|work| {
        assert_eq!(
            backup_workspace_malvin_config_workspace_if_present(work).unwrap(),
            MalvinConfigWorkspaceBackup::Missing
        );
    });
}

#[test]
fn malvin_config_workspace_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::create_dir_all(work.join(".malvin")).unwrap();
        let cfg = work.join(MALVIN_CONFIG_REL);
        std::fs::write(&cfg, "ORIGINAL\n").unwrap();
        let backup = backup_workspace_malvin_config_workspace_if_present(work).unwrap();
        std::fs::write(&cfg, "MODIFIED\n").unwrap();
        restore_workspace_malvin_config_workspace_backup(work, &backup).unwrap();
        assert_eq!(std::fs::read_to_string(&cfg).unwrap(), "ORIGINAL\n");
    });
}

#[test]
fn malvin_config_workspace_backup_missing_removes_agent_created_file() {
    with_isolated_home(|work| {
        std::fs::create_dir_all(work.join(".malvin")).unwrap();
        let backup = backup_workspace_malvin_config_workspace_if_present(work).unwrap();
        let cfg = work.join(MALVIN_CONFIG_REL);
        std::fs::write(&cfg, "CREATED\n").unwrap();
        restore_workspace_malvin_config_workspace_backup(work, &backup).unwrap();
        assert!(!cfg.exists());
    });
}
