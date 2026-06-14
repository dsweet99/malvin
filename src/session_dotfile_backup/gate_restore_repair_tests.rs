use super::repair_clamp_damaged_dotfiles_on_disk;
use crate::session_dotfile_backup::gate_restore_merge::kissconfig_low_coverage_threshold;

fn seed_clamp_damaged_workspace(work: &std::path::Path) {
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss\n").expect("checks");
    std::fs::write(
        work.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 0\n",
    )
    .expect("kissconfig");
    std::fs::write(work.join("lib.rs"), "fn main() {}\n").expect("source");
    std::fs::write(work.join("Cargo.toml"), "[package]\nname = \"t\"\n").expect("cargo");
}

#[test]
fn repair_leaves_empty_checks_file_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "").expect("checks");

    repair_clamp_damaged_dotfiles_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(work.join(".malvin/checks")).expect("checks");
    assert!(checks.is_empty());
}

#[test]
fn repair_ignores_non_utf8_checks_that_are_not_bare_kiss() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), b"\xff\xfe").expect("checks");

    repair_clamp_damaged_dotfiles_on_disk(work).expect("repair");

    let checks = std::fs::read(work.join(".malvin/checks")).expect("checks");
    assert_eq!(checks, b"\xff\xfe");
}

#[test]
fn repair_materializes_missing_checks_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join("Cargo.toml"), "[package]\nname = \"t\"\n").expect("cargo");
    std::fs::write(work.join("lib.rs"), "fn main() {}\n").expect("source");

    repair_clamp_damaged_dotfiles_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(work.join(".malvin/checks")).expect("checks");
    assert!(checks.contains("kiss check"));
}

#[test]
fn repair_leaves_valid_checks_and_kissconfig_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    std::fs::write(
        work.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 90\n",
    )
    .expect("kissconfig");

    repair_clamp_damaged_dotfiles_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(work.join(".malvin/checks")).expect("checks");
    assert_eq!(checks, "kiss check\n");
    let kissconfig = std::fs::read_to_string(work.join(".kissconfig")).expect("kissconfig");
    assert_eq!(kissconfig, "[gate]\ntest_coverage_threshold = 90\n");
}

#[test]
fn repair_clamp_damaged_dotfiles_on_disk_fixes_bare_kiss_and_low_threshold() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    seed_clamp_damaged_workspace(work);

    repair_clamp_damaged_dotfiles_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(work.join(".malvin/checks")).expect("checks");
    assert!(checks.contains("kiss check"));
    assert_ne!(checks.trim(), "kiss");
    let kissconfig = std::fs::read_to_string(work.join(".kissconfig")).expect("kissconfig");
    assert!(kissconfig.contains("test_coverage_threshold = 90"));
    assert!(!kissconfig_low_coverage_threshold(kissconfig.as_bytes()));
}

#[test]
fn sanitize_bundle_fixes_both_poisoned_merge_slots() {
    use crate::session_dotfile_backup::gate_restore_merge::{
        merge_and_sanitize_for_gate_restore, merge_for_gate_restore,
    };
    use crate::session_dotfile_backup::{
        DotfileBackupPayload, DotfileBackupState, GitignoreBackup, SessionDotfileBackups,
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::write(work.join("Cargo.toml"), "[package]\nname = \"t\"\n").expect("cargo");
    std::fs::write(work.join("lib.rs"), "fn main() {}\n").expect("source");

    let poisoned = |bytes: &[u8]| {
        DotfileBackupState::Present(DotfileBackupPayload {
            backup_path: work.join("slot"),
            bytes: bytes.to_vec(),
        })
    };
    let anchor = SessionDotfileBackups {
        kissconfig: poisoned(b"[gate]\ntest_coverage_threshold = 0\n"),
        malvin_checks: poisoned(b"kiss\n"),
        kissignore: DotfileBackupState::Missing,
        malvin_config: DotfileBackupState::Missing,
        gitignore: GitignoreBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    };
    let progress = anchor.clone();
    let merged = merge_for_gate_restore(&anchor, &progress);
    let DotfileBackupState::Present(ref checks) = merged.malvin_checks else {
        panic!("expected checks present");
    };
    assert_eq!(checks.bytes, b"kiss\n");

    let sanitized = merge_and_sanitize_for_gate_restore(&anchor, &progress, work);
    let DotfileBackupState::Present(ref checks) = sanitized.malvin_checks else {
        panic!("expected checks present");
    };
    assert!(String::from_utf8_lossy(&checks.bytes).contains("kiss check"));
    let DotfileBackupState::Present(ref kissconfig) = sanitized.kissconfig else {
        panic!("expected kissconfig present");
    };
    assert!(String::from_utf8_lossy(&kissconfig.bytes).contains("test_coverage_threshold = 90"));
}

#[test]
fn repair_recreates_empty_home_malvin_config_from_template() {
    crate::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"").expect("empty home config");
        repair_clamp_damaged_dotfiles_on_disk(work).expect("repair");
        let text = std::fs::read_to_string(&cfg).expect("read home config");
        assert!(text.contains("mem_limit_gb"));
        assert!(text.contains("[agent]"));
    });
}

#[test]
fn sanitize_bundle_replaces_empty_home_malvin_config_with_template() {
    use crate::session_dotfile_backup::sanitize_clamp_damaged_dotfiles_in_bundle;
    use crate::session_dotfile_backup::{
        DotfileBackupPayload, DotfileBackupState, GitignoreBackup, SessionDotfileBackups,
    };

    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    let poisoned = |bytes: &[u8]| {
        DotfileBackupState::Present(DotfileBackupPayload {
            backup_path: work.join("slot"),
            bytes: bytes.to_vec(),
        })
    };
    let mut bundle = SessionDotfileBackups {
        kissconfig: DotfileBackupState::Missing,
        malvin_checks: DotfileBackupState::Missing,
        kissignore: DotfileBackupState::Missing,
        malvin_config: poisoned(b""),
        gitignore: GitignoreBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    };
    sanitize_clamp_damaged_dotfiles_in_bundle(&mut bundle, work);
    let DotfileBackupState::Present(ref cfg) = bundle.malvin_config else {
        panic!("expected home config present");
    };
    let text = String::from_utf8_lossy(&cfg.bytes);
    assert!(text.contains("mem_limit_gb"));
    assert!(text.contains("[agent]"));
}
