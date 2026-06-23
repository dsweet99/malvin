//! External kiss witnesses for [`super::wrappers`] and [`super::alloc`].

#[test]
fn kiss_witness_session_dotfile_wrappers_and_alloc() {
    let _ = super::alloc::random_backup_id;
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let _ = super::wrappers::backup_workspace_malvin_config_if_present(work);
    let _ = super::wrappers::backup_workspace_malvin_config_if_present_with_id(work, |n| {
        format!("mc{n}")
    });
    let _ = super::wrappers::backup_workspace_malvin_config_workspace_if_present(work);
    let _ = super::wrappers::backup_workspace_malvin_config_workspace_if_present_with_id(work, |n| {
        format!("mws{n}")
    });
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/config"), "x\n").expect("write config");
    if let Ok(backup) = super::wrappers::backup_workspace_malvin_config_if_present(work) {
        let _ = super::wrappers::restore_workspace_malvin_config_backup(work, &backup);
    }
    if let Ok(backup) = super::wrappers::backup_workspace_malvin_config_workspace_if_present(work) {
        let _ = super::wrappers::restore_workspace_malvin_config_workspace_backup(work, &backup);
    }
    let _ = super::wrappers::restore_workspace_malvin_config_backup;
    let _ = super::wrappers::restore_workspace_malvin_config_workspace_backup;
}
