#[test]
fn kiss_witness_session_dotfile_wrappers_and_alloc() {
    let _ = super::alloc::random_backup_id;
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let _ = super::wrappers::backup_workspace_malvin_checks_if_present(work);
    let _ = super::wrappers::backup_workspace_malvin_checks_if_present_with_id(work, |n| {
        format!("mc{n}")
    });
    if let Ok(backup) = super::wrappers::backup_workspace_malvin_checks_if_present(work) {
        let _ = super::wrappers::restore_workspace_malvin_checks_backup(work, &backup);
    }
    let _ = super::wrappers::restore_workspace_malvin_checks_backup;
}
