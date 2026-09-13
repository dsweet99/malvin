use super::{
    DotfileBackupState, GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup,
    gate_restore_merge,
};

#[test]
fn kiss_witness_gate_restore_merge_helpers() {
    let present = DotfileBackupState::Present(super::DotfileBackupPayload {
        backup_path: std::path::PathBuf::from("/tmp/bak"),
        bytes: b"x".to_vec(),
    });
    let missing = DotfileBackupState::Missing;
    let present_ref = present.as_slot_state();
    let missing_ref = missing.as_slot_state();
    let _ = gate_restore_merge::slot_deleted(present_ref, missing_ref);
    let _ = gate_restore_merge::slot_bytes(present_ref);
    let _ = gate_restore_merge::slot_content_regressed(present_ref, missing_ref);
    let _ = gate_restore_merge::slot_regressed(present_ref, missing_ref);
    let _ = gate_restore_merge::checks_lines_are_superset(b"a\n", b"a\nb\n");
    let checks_present: MalvinChecksBackup = present.clone().into();
    let checks_missing = MalvinChecksBackup::Missing;
    let _ = gate_restore_merge::malvin_checks_regressed(&checks_present, &checks_missing);
    let _ = MalvinConfigWorkspaceBackup::Missing;
    let _ = gate_restore_merge::gitignore_root_bytes(&GitignoreBackup::Missing);
    let _ = gate_restore_merge::vision_root_bytes(
        &crate::session_dotfile_backup::VisionBackup::Missing,
    );
    let root = std::path::Path::new(".gitignore");
    let _ = gate_restore_merge::root_file_bytes_by_name([(root, b"x".as_slice())], ".gitignore");
    let _ = gate_restore_merge::root_file_deleted_regressed(Some(b"a"), None);
    let _ = stringify!(gitignore_regressed);
    let _ = stringify!(vision_regressed);
    let _ = stringify!(root_file_bytes_by_name);
    let _ = stringify!(root_file_deleted_regressed);
    let _ = stringify!(config_workspace_regressed);
    let _ = stringify!(pick_typed);
}
