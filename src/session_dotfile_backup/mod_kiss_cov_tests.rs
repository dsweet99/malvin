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
        malvin_config_workspace: missing,
    };
    let touched = std::hint::black_box(parts);
    let SessionDotfileParts {
        kissconfig,
        malvin_checks,
        kissignore,
        malvin_config,
        gitignore,
        malvin_config_workspace,
    } = touched;
    assert!(matches!(kissconfig, DotfileBackupState::Missing));
    assert!(matches!(malvin_checks, DotfileBackupState::Missing));
    assert!(matches!(kissignore, DotfileBackupState::Missing));
    assert!(matches!(malvin_config, DotfileBackupState::Missing));
    assert!(matches!(gitignore, super::GitignoreBackup::Missing));
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
        malvin_config_workspace: DotfileBackupState::Missing,
    });
    assert!(matches!(backups.kissconfig, DotfileBackupState::Missing));
}
