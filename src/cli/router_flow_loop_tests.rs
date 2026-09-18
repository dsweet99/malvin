use super::restore_router_iteration_dotfiles;
use malvin::session_dotfile_backup::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup, SessionDotfileBackups,
    VisionBackup, VisionFileBackup,
};

#[test]
fn restore_router_iteration_keeps_agent_vision_edits() {
    malvin::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let work = tmp.path();
        std::fs::write(work.join("VISION.md"), "baseline prine:\n").expect("write");
        let anchor = SessionDotfileBackups::snapshot(work).expect("anchor");
        std::fs::write(
            work.join("VISION.md"),
            "- `rpi:` models should look basically the same as `cursor:` models.\n",
        )
        .expect("edit");
        let merged = restore_router_iteration_dotfiles(work, &anchor).expect("restore");
        let text = std::fs::read_to_string(work.join("VISION.md")).expect("read");
        assert!(
            text.contains("`rpi:`") && !text.contains("prine:"),
            "expected agent VISION edit kept, got: {text:?}"
        );
        assert!(matches!(merged.vision, VisionBackup::Present { .. }));
    });
}

#[test]
fn restore_router_iteration_restores_deleted_vision() {
    malvin::test_utils::with_isolated_home(|_| {
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
        malvin_checks: MalvinChecksBackup::Missing,
        gitignore: GitignoreBackup::Missing,
        vision: VisionBackup::Missing,
        malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
    };
    let _ = matches!(empty.vision, VisionBackup::Missing);
    let _ = VisionFileBackup {
        rel: std::path::PathBuf::from("VISION.md"),
        bytes: b"x".to_vec(),
    };
}

#[test]
fn exit_gates_failed_outranks_finalize_error() {
    use super::router_flow_loop_decide::prefer_exit_gates_over_acp;
    use super::RouterLoopDecision;
    let gates = Some(RouterLoopDecision::ExitGatesFailed(
        "gate detail".to_string(),
    ));
    let acp_err = Err("finalize failed".to_string());
    let decided = prefer_exit_gates_over_acp(&acp_err, gates);
    assert!(matches!(
        decided,
        RouterLoopDecision::ExitGatesFailed(detail) if detail == "gate detail"
    ));
    let continued = prefer_exit_gates_over_acp(&acp_err, Some(RouterLoopDecision::Continue));
    assert!(matches!(continued, RouterLoopDecision::Exit));
    let proceed = prefer_exit_gates_over_acp(&Ok(()), Some(RouterLoopDecision::Continue));
    assert!(matches!(proceed, RouterLoopDecision::Continue));
    let stop = prefer_exit_gates_over_acp(&Ok(()), None);
    assert!(matches!(stop, RouterLoopDecision::Exit));
}
