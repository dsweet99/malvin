use crate::session_dotfile_backup::gate_restore_merge::merge_for_gate_restore;
use crate::session_dotfile_backup::{
    DotfileBackupPayload, DotfileBackupState, GitignoreBackup, GitignoreFileBackup,
    SessionDotfileBackups,
};

fn present(bytes: &[u8]) -> DotfileBackupState {
    DotfileBackupState::Present(DotfileBackupPayload {
        backup_path: std::path::PathBuf::from("/tmp/test"),
        bytes: bytes.to_vec(),
    })
}

fn gitignore_present(bytes: &[u8]) -> GitignoreBackup {
    GitignoreBackup::Present {
        backup_root: std::path::PathBuf::from("/tmp/test"),
        files: vec![GitignoreFileBackup {
            rel: std::path::PathBuf::from(".gitignore"),
            bytes: bytes.to_vec(),
        }],
    }
}

fn bundle_with(gitignore: GitignoreBackup, checks: DotfileBackupState) -> SessionDotfileBackups {
    SessionDotfileBackups {
        malvin_checks: checks,
        gitignore,
        vision: crate::session_dotfile_backup::VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    }
}

#[test]
fn merge_rejects_deleted_vision() {
    let vision_present = |bytes: &[u8]| crate::session_dotfile_backup::VisionBackup::Present {
        backup_root: std::path::PathBuf::from("/tmp/test"),
        files: vec![crate::session_dotfile_backup::VisionFileBackup {
            rel: std::path::PathBuf::from("VISION.md"),
            bytes: bytes.to_vec(),
        }],
    };
    let mut anchor = bundle_with(GitignoreBackup::Missing, DotfileBackupState::Missing);
    anchor.vision = vision_present(b"baseline\n");
    let mut progress = anchor.clone();
    progress.vision = crate::session_dotfile_backup::VisionBackup::Missing;
    let merged = merge_for_gate_restore(&anchor, &progress);
    assert!(matches!(
        merged.vision,
        crate::session_dotfile_backup::VisionBackup::Present { .. }
    ));
}

#[test]
fn merge_rejects_deleted_gitignore() {
    let anchor = bundle_with(gitignore_present(b"baseline\n"), present(b"make lint\n"));
    let progress = bundle_with(GitignoreBackup::Missing, present(b"make lint\n"));
    let merged = merge_for_gate_restore(&anchor, &progress);
    assert!(matches!(merged.gitignore, GitignoreBackup::Present { .. }));
    assert!(matches!(
        merged.malvin_checks,
        DotfileBackupState::Present(_)
    ));
}

#[test]
fn merge_rejects_tampered_malvin_checks() {
    let anchor = bundle_with(GitignoreBackup::Missing, present(b"make lint\n"));
    let progress = bundle_with(GitignoreBackup::Missing, present(b"TAMPERED\n"));
    let merged = merge_for_gate_restore(&anchor, &progress);
    let DotfileBackupState::Present(ref payload) = merged.malvin_checks else {
        panic!("expected malvin_checks present");
    };
    assert_eq!(payload.bytes, b"make lint\n");
}

#[test]
fn merge_keeps_agent_edited_vision_content() {
    let vision_present = |bytes: &[u8]| crate::session_dotfile_backup::VisionBackup::Present {
        backup_root: std::path::PathBuf::from("/tmp/test"),
        files: vec![crate::session_dotfile_backup::VisionFileBackup {
            rel: std::path::PathBuf::from("VISION.md"),
            bytes: bytes.to_vec(),
        }],
    };
    let mut anchor = bundle_with(GitignoreBackup::Missing, DotfileBackupState::Missing);
    anchor.vision = vision_present(b"baseline\n");
    let mut progress = anchor.clone();
    progress.vision = vision_present(b"improved vision\n");
    let merged = merge_for_gate_restore(&anchor, &progress);
    let crate::session_dotfile_backup::VisionBackup::Present { files, .. } = merged.vision else {
        panic!("expected vision present");
    };
    assert_eq!(files[0].bytes, b"improved vision\n");
}

#[test]
fn merge_keeps_agent_edited_gitignore_content() {
    let anchor = bundle_with(gitignore_present(b"baseline\n"), present(b"make lint\n"));
    let progress = bundle_with(
        gitignore_present(b"baseline\nextra/\n"),
        present(b"make lint\n"),
    );
    let merged = merge_for_gate_restore(&anchor, &progress);
    let GitignoreBackup::Present { files, .. } = merged.gitignore else {
        panic!("expected gitignore present");
    };
    assert_eq!(files[0].bytes, b"baseline\nextra/\n");
}

#[test]
fn merge_keeps_progress_when_gitignore_present_without_root_file() {
    let nested_only = GitignoreBackup::Present {
        backup_root: std::path::PathBuf::from("/tmp/test"),
        files: vec![GitignoreFileBackup {
            rel: std::path::PathBuf::from("pkg/.gitignore"),
            bytes: b"pkg\n".to_vec(),
        }],
    };
    let anchor = bundle_with(GitignoreBackup::Missing, present(b"make lint\n"));
    let progress = bundle_with(nested_only, present(b"make lint\n"));
    let merged = merge_for_gate_restore(&anchor, &progress);
    let GitignoreBackup::Present { files, .. } = merged.gitignore else {
        panic!("expected present");
    };
    assert_eq!(files[0].rel, std::path::PathBuf::from("pkg/.gitignore"));
}

#[test]
fn merge_keeps_progress_when_vision_present_without_root_file() {
    let nested_only = crate::session_dotfile_backup::VisionBackup::Present {
        backup_root: std::path::PathBuf::from("/tmp/test"),
        files: vec![crate::session_dotfile_backup::VisionFileBackup {
            rel: std::path::PathBuf::from("pkg/VISION.md"),
            bytes: b"pkg\n".to_vec(),
        }],
    };
    let mut anchor = bundle_with(GitignoreBackup::Missing, DotfileBackupState::Missing);
    anchor.vision = crate::session_dotfile_backup::VisionBackup::Missing;
    let mut progress = anchor.clone();
    progress.vision = nested_only;
    let merged = merge_for_gate_restore(&anchor, &progress);
    let crate::session_dotfile_backup::VisionBackup::Present { files, .. } = merged.vision else {
        panic!("expected vision present");
    };
    assert_eq!(files[0].rel, std::path::PathBuf::from("pkg/VISION.md"));
}
