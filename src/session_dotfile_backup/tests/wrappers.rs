use crate::artifacts::backup_workspace_malvin_checks_if_present;
use crate::test_support::test_utils::with_isolated_home;

#[test]
fn wrapper_checks_backup_skips_when_file_missing() {
    with_isolated_home(|work| {
        assert!(matches!(
            backup_workspace_malvin_checks_if_present(work).unwrap(),
            crate::artifacts::MalvinChecksBackup::Missing
        ));
    });
}
