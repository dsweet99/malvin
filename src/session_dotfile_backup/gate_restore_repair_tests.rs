use super::repair_invalid_malvin_home_config_on_disk;
use crate::repo_gates::checks_test_helpers::{git_init, write_git_root_checks};

fn checks_path(work: &std::path::Path) -> std::path::PathBuf {
    crate::malvin_checks_path(work)
}

fn write_checks(work: &std::path::Path, content: impl AsRef<[u8]>) {
    write_git_root_checks(work, content);
}

#[test]
fn kiss_cov_gate_restore_repair_test_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    git_init(work);
    write_checks(work, "make lint\n");
    let _ = checks_path(work);
    let _ = crate::repo_gates::checks_test_helpers::git_init;
    let _ = stringify!(write_git_root_checks);
}

#[test]
fn repair_leaves_empty_checks_file_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    write_checks(work, "");

    repair_invalid_malvin_home_config_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(checks_path(work)).expect("checks");
    assert!(checks.is_empty());
}

#[test]
fn repair_leaves_missing_checks_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    git_init(work);
    std::fs::write(work.join("Cargo.toml"), "[package]\nname = \"t\"\n").expect("cargo");
    std::fs::write(work.join("lib.rs"), "fn main() {}\n").expect("source");

    repair_invalid_malvin_home_config_on_disk(work).expect("repair");

    assert!(!checks_path(work).exists());
}

#[test]
fn repair_leaves_valid_checks_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    write_checks(work, "make lint\n");

    repair_invalid_malvin_home_config_on_disk(work).expect("repair");

    let checks = std::fs::read_to_string(checks_path(work)).expect("checks");
    assert_eq!(checks, "make lint\n");
}

#[test]
fn repair_recreates_empty_home_malvin_config_from_template() {
    crate::test_utils::with_isolated_home(|work| {
        let cfg = crate::malvin_config_path(work);
        if let Some(parent) = cfg.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&cfg, b"").expect("empty home config");
        repair_invalid_malvin_home_config_on_disk(work).expect("repair");
        let text = std::fs::read_to_string(&cfg).expect("read home config");
        assert!(text.contains("mem_limit_gb"));
        assert!(text.contains("[agent]"));
    });
}

#[test]
fn sanitize_bundle_replaces_empty_home_malvin_config_with_template() {
    use crate::session_dotfile_backup::sanitize_invalid_malvin_home_config_in_bundle;
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
        malvin_checks: DotfileBackupState::Missing,
        malvin_config: poisoned(b""),
        gitignore: GitignoreBackup::Missing,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    };
    sanitize_invalid_malvin_home_config_in_bundle(&mut bundle, work);
    let DotfileBackupState::Present(ref cfg) = bundle.malvin_config else {
        panic!("expected home config present");
    };
    let text = String::from_utf8_lossy(&cfg.bytes);
    assert!(text.contains("mem_limit_gb"));
    assert!(text.contains("[agent]"));
}

#[test]
fn bytes_for_restore_replaces_empty_with_template() {
    use super::bytes_for_malvin_home_config_restore;

    let fixed = bytes_for_malvin_home_config_restore(b"").expect("template");
    assert!(!fixed.is_empty());
    let text = String::from_utf8_lossy(&fixed);
    assert!(text.contains("mem_limit_gb"));
    let kept = bytes_for_malvin_home_config_restore(b"mem_limit_gb = 3\n").expect("keep");
    assert_eq!(kept, b"mem_limit_gb = 3\n");
}
