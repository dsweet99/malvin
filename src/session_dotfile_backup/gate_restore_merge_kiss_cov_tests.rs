//! External kiss witnesses for [`super::gate_restore_merge`] privates.

use super::{DotfileBackupState, gate_restore_merge};

#[test]
fn kiss_witness_gate_restore_merge_helpers() {
    let present = DotfileBackupState::Present(super::DotfileBackupPayload {
        backup_path: std::path::PathBuf::from("/tmp/bak"),
        bytes: b"x".to_vec(),
    });
    let missing = DotfileBackupState::Missing;
    let _ = gate_restore_merge::slot_deleted(&present, &missing);
    let _ = gate_restore_merge::kissignore_agent_created(&missing, &present);
    let _ = gate_restore_merge::slot_bytes(&present);
    let _ = gate_restore_merge::slot_content_regressed(&present, &missing);
    let _ = gate_restore_merge::kissconfig_threshold_regressed(&present, &missing);
    let _ = gate_restore_merge::kissconfig_repaired_clamp_damage(&present, &missing);
    let _ = gate_restore_merge::malvin_checks_repaired_clamp_damage(&present, &missing);
    let _ = gate_restore_merge::kissconfig_regressed(&present, &missing);
    let _ = gate_restore_merge::slot_regressed(&present, &missing);
    let _ = gate_restore_merge::checks_lines_are_superset(b"a\n", b"a\nb\n");
    let _ = gate_restore_merge::malvin_checks_regressed(&present, &missing);
    let _ = gate_restore_merge::kissconfig_low_coverage_threshold;
}
