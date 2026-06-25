//! External kiss witnesses for [`super`] session dotfile types.

use super::{
    DotfileBackupPayload, DotfileBackupState, SessionDotfileParts, SessionDotfileBackups,
};

#[test]
fn kiss_cov_dotfile_backup_payload_construct_destructure() {
    let payload = DotfileBackupPayload {
        backup_path: std::path::PathBuf::from("/tmp/backup/.kissconfig"),
        bytes: b"threshold = 90\n".to_vec(),
    };
    let touched = std::hint::black_box(payload);
    let DotfileBackupPayload { backup_path, bytes } = touched;
    assert!(backup_path.ends_with(".kissconfig"));
    assert_eq!(bytes, b"threshold = 90\n");
}

#[test]
fn kiss_cov_session_dotfile_parts_construct_destructure() {
    let missing = DotfileBackupState::Missing;
    let parts = SessionDotfileParts {
        kissconfig: missing.clone(),
        malvin_checks: missing.clone(),
        kissignore: missing.clone(),
        malvin_config: missing.clone(),
        gitignore: super::GitignoreBackup::Missing,
        vision: super::VisionBackup::Missing,
        malvin_config_workspace: missing,
    };
    let touched = std::hint::black_box(parts);
    let SessionDotfileParts {
        kissconfig,
        malvin_checks,
        kissignore,
        malvin_config,
        gitignore,
        vision,
        malvin_config_workspace,
    } = touched;
    assert!(matches!(kissconfig, DotfileBackupState::Missing));
    assert!(matches!(malvin_checks, DotfileBackupState::Missing));
    assert!(matches!(kissignore, DotfileBackupState::Missing));
    assert!(matches!(malvin_config, DotfileBackupState::Missing));
    assert!(matches!(gitignore, super::GitignoreBackup::Missing));
    assert!(matches!(vision, super::VisionBackup::Missing));
    assert!(matches!(
        malvin_config_workspace,
        DotfileBackupState::Missing
    ));
    let backups = SessionDotfileBackups::from_parts(SessionDotfileParts {
        kissconfig: DotfileBackupState::Missing,
        malvin_checks: DotfileBackupState::Missing,
        kissignore: DotfileBackupState::Missing,
        malvin_config: DotfileBackupState::Missing,
        gitignore: super::GitignoreBackup::Missing,
        vision: super::VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    });
    assert!(matches!(backups.kissconfig, DotfileBackupState::Missing));
}

#[test]
fn kiss_cov_gitignore_file_backup_construct_destructure() {
    let file = super::gitignore_tree::GitignoreFileBackup {
        rel: std::path::PathBuf::from(".gitignore"),
        bytes: b"target/\n".to_vec(),
    };
    let super::gitignore_tree::GitignoreFileBackup { rel, bytes } = file;
    assert_eq!(rel, std::path::PathBuf::from(".gitignore"));
    assert_eq!(bytes, b"target/\n");
}

#[test]
fn kiss_cov_vision_file_backup_construct_destructure() {
    let file = super::vision_tree::VisionFileBackup {
        rel: std::path::PathBuf::from("VISION.md"),
        bytes: b"# Vision\n".to_vec(),
    };
    let super::vision_tree::VisionFileBackup { rel, bytes } = file;
    assert_eq!(rel, std::path::PathBuf::from("VISION.md"));
    assert_eq!(bytes, b"# Vision\n");
}

#[test]
fn kiss_cov_write_merged_default_malvin_config() {
    let _ = super::slots_kiss_cov_shared::write_merged_default_malvin_config;

    crate::test_utils::with_isolated_home(|work| {
        let cfg_path = crate::malvin_config_path(work);
        if let Some(parent) = cfg_path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir home config parent");
        }
        super::slots_kiss_cov_shared::write_merged_default_malvin_config(&cfg_path);
        assert!(cfg_path.is_file(), "default config should be written");
        let content = std::fs::read_to_string(&cfg_path).expect("read config");
        assert!(content.ends_with('\n'), "config should end with newline");
        assert!(
            content.contains("[agent]") || content.contains("memory"),
            "merged template keys should appear"
        );
    });
}

#[test]
fn kiss_cov_slots_kiss_cov_shared_fn_refs() {
    let _ = super::slots_kiss_cov_shared::dotfile_spec_row_field_count;
    let _ = super::alloc::random_backup_id;
}
