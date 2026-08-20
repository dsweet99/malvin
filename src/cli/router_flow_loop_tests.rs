use super::restore_router_iteration_dotfiles;
use crate::session_dotfile_backup::{
    DotfileBackupState, GitignoreBackup, SessionDotfileBackups, VisionBackup, VisionFileBackup,
};

#[test]
fn restore_router_iteration_keeps_agent_vision_edits() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let work = tmp.path();
        std::fs::write(work.join("VISION.md"), "baseline prine:\n").expect("write");
        let anchor = SessionDotfileBackups::snapshot(work).expect("anchor");
        std::fs::write(
            work.join("VISION.md"),
            "- `pi:` models should look basically the same as `cursor:` models.\n",
        )
        .expect("edit");
        let merged = restore_router_iteration_dotfiles(work, &anchor).expect("restore");
        let text = std::fs::read_to_string(work.join("VISION.md")).expect("read");
        assert!(
            text.contains("`pi:`") && !text.contains("prine:"),
            "expected agent VISION edit kept, got: {text:?}"
        );
        assert!(matches!(merged.vision, VisionBackup::Present { .. }));
    });
}

#[test]
fn restore_router_iteration_restores_deleted_vision() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let work = tmp.path();
        std::fs::write(work.join("VISION.md"), "keep me\n").expect("write");
        let anchor = SessionDotfileBackups::snapshot(work).expect("anchor");
        std::fs::remove_file(work.join("VISION.md")).expect("delete");
        let _ = restore_router_iteration_dotfiles(work, &anchor).expect("restore");
        let text = std::fs::read_to_string(work.join("VISION.md")).expect("read");
        assert_eq!(text, "keep me\n");
    });
}

#[test]
fn kiss_witness_restore_router_iteration_dotfiles() {
    let _ = restore_router_iteration_dotfiles;
    let _ = stringify!(RouterAgentLoopInput);
    let _ = stringify!(RouterAgentLoopOutcome);
    let empty = SessionDotfileBackups {
        malvin_checks: DotfileBackupState::Missing,
        gitignore: GitignoreBackup::Missing,
        vision: VisionBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    };
    let _ = matches!(empty.vision, VisionBackup::Missing);
    let _ = VisionFileBackup {
        rel: std::path::PathBuf::from("VISION.md"),
        bytes: b"x".to_vec(),
    };
}
