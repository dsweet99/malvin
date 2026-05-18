#[test]
fn kiss_stringify_session_dotfiles_units() {
    let _ = stringify!(crate::session_dotfile_backup::DotfileBackupState);
    let _ = stringify!(crate::session_dotfile_backup::SessionDotfileBackups);
    let _ = stringify!(crate::session_dotfile_backup::backup_workspace_kissconfig_if_present);
    let _ = stringify!(crate::session_dotfile_backup::restore_workspace_kissconfig_backup);
    let _ =
        stringify!(crate::session_dotfile_backup::backup_workspace_malvin_checks_if_present);
    let _ = stringify!(crate::session_dotfile_backup::restore_workspace_malvin_checks_backup);
    let _ = stringify!(crate::session_dotfile_backup::backup_workspace_kissignore_if_present);
    let _ = stringify!(crate::session_dotfile_backup::restore_workspace_kissignore_backup);
    let _ = stringify!(crate::session_dotfile_backup::allocate_backup_dir);
    let _ = stringify!(crate::session_dotfile_backup::remove_if_exists);
}
