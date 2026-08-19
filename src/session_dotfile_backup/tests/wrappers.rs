use crate::artifacts::{
    MalvinConfigWorkspaceBackup, backup_workspace_malvin_config_workspace_if_present,
    backup_workspace_malvin_config_workspace_if_present_with_id,
    restore_workspace_malvin_config_workspace_backup,
};
use crate::test_utils::with_isolated_home;

fn write_workspace_config(work: &std::path::Path, body: &str) {
    std::fs::create_dir_all(work.join(".malvin")).unwrap();
    std::fs::write(work.join(crate::MALVIN_CONFIG_REL), body).unwrap();
}

#[test]
fn wrapper_workspace_config_backup_and_restore_round_trip() {
    with_isolated_home(|work| {
        write_workspace_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_workspace_if_present(work).unwrap();
        let MalvinConfigWorkspaceBackup::Present(payload) = &backup else {
            panic!("expected workspace config backup");
        };
        assert!(payload.backup_path.is_file());
        write_workspace_config(work, "MODIFIED\n");
        restore_workspace_malvin_config_workspace_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(crate::MALVIN_CONFIG_REL)).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn wrapper_workspace_config_backup_with_id_retries_collision() {
    with_isolated_home(|work| {
        let dir = crate::workspace_paths::snapshot_category_dir("malvin_config_workspace");
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();
        write_workspace_config(work, "ORIGINAL\n");
        let backup = backup_workspace_malvin_config_workspace_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();
        let MalvinConfigWorkspaceBackup::Present(payload) = &backup else {
            panic!("expected backup");
        };
        assert!(payload.backup_path.starts_with(dir.join("bbbbb")));
    });
}
