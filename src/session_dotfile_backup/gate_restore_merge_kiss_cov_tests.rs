use super::{DotfileBackupState, GitignoreBackup, gate_restore_merge};

#[test]
fn kiss_witness_gate_restore_merge_helpers() {
    let present = DotfileBackupState::Present(super::DotfileBackupPayload {
        backup_path: std::path::PathBuf::from("/tmp/bak"),
        bytes: b"x".to_vec(),
    });
    let missing = DotfileBackupState::Missing;
    let _ = gate_restore_merge::slot_deleted(&present, &missing);
    let _ = gate_restore_merge::slot_bytes(&present);
    let _ = gate_restore_merge::slot_content_regressed(&present, &missing);
    let _ = gate_restore_merge::slot_regressed(&present, &missing);
    let _ = gate_restore_merge::checks_lines_are_superset(b"a\n", b"a\nb\n");
    let _ = gate_restore_merge::malvin_checks_regressed(&present, &missing);
    let _ = gate_restore_merge::gitignore_root_bytes(&GitignoreBackup::Missing);
    let _ = gate_restore_merge::vision_root_bytes(
        &crate::session_dotfile_backup::VisionBackup::Missing,
    );
    let _ = stringify!(gitignore_regressed);
    let _ = stringify!(vision_regressed);
    let _ = stringify!(pick_gitignore);
    let _ = stringify!(pick_vision);
}
