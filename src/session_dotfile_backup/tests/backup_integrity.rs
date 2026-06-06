use crate::artifacts::{
    backup_workspace_kissconfig_if_present, restore_workspace_kissconfig_backup,
    KissConfigBackup,
};
use crate::test_utils::with_isolated_home;

/// Regression: an agent running as the same UID can overwrite the on-disk snapshot under
/// `~/.malvin` after backup; restore must still write the bytes captured at snapshot time.
#[test]
fn poisoned_disk_snapshot_does_not_change_restored_workspace_content() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".kissconfig"), "KISS=ORIGINAL\n").unwrap();
        let backup = backup_workspace_kissconfig_if_present(work).unwrap();
        let KissConfigBackup::Present(payload) = &backup else {
            panic!("expected backup payload");
        };

        std::fs::write(&payload.backup_path, "KISS=POISONED\n").unwrap();
        std::fs::write(work.join(".kissconfig"), "KISS=AGENT\n").unwrap();

        restore_workspace_kissconfig_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".kissconfig")).unwrap(),
            "KISS=ORIGINAL\n"
        );
    });
}
