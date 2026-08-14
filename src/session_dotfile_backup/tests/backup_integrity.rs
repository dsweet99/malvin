use crate::artifacts::{
    backup_workspace_malvin_checks_if_present, restore_workspace_malvin_checks_backup,
    MalvinChecksBackup,
};
use crate::test_utils::with_isolated_home;

#[test]
fn poisoned_disk_snapshot_does_not_change_restored_workspace_content() {
    with_isolated_home(|work| {
        std::fs::create_dir_all(work.join(".malvin")).unwrap();
        std::fs::write(work.join(".malvin/checks"), "make lint\n").unwrap();
        let backup = backup_workspace_malvin_checks_if_present(work).unwrap();
        let MalvinChecksBackup::Present(payload) = &backup else {
            panic!("expected backup payload");
        };

        std::fs::write(&payload.backup_path, "make lint\nPOISONED\n").unwrap();
        std::fs::write(work.join(".malvin/checks"), "make lint\nAGENT\n").unwrap();

        restore_workspace_malvin_checks_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".malvin/checks")).unwrap(),
            "make lint\n"
        );
    });
}
