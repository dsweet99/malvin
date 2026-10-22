use super::restore_router_iteration_dotfiles;
use malvin::session_dotfile_backup::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup, SessionDotfileBackups,
    VisionBackup, VisionFileBackup,
};

#[test]
fn restore_router_iteration_keeps_agent_vision_edits() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
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
    malvin::test_support::test_utils::with_isolated_home(|_| {
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
    let _ = VisionFileBackup::new(std::path::PathBuf::from("VISION.md"), b"x".to_vec());
}

struct LoopExitCase {
    done: bool,
    request_open: bool,
    gates: bool,
    agent_loop: usize,
    max_loops: usize,
}

fn decide_loop_case(
    artifacts: &malvin::artifacts::RunArtifacts,
    backups: &malvin::artifacts::SessionDotfileBackups,
    case: LoopExitCase,
) -> super::RouterLoopDecision {
    super::decide_router_loop_exit(super::RouterLoopExitInput {
        artifacts,
        backups,
        done: case.done,
        request_open: case.request_open,
        gates: case.gates,
        agent_loop: case.agent_loop,
        max_loops: case.max_loops,
    })
}

fn router_loop_artifacts(
    work: &std::path::Path,
) -> (
    malvin::artifacts::RunArtifacts,
    malvin::artifacts::SessionDotfileBackups,
) {
    let artifacts =
        malvin::artifacts::create_run_artifacts_from_text("router loop exit", Some(work))
            .expect("artifacts");
    let backups = malvin::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot");
    (artifacts, backups)
}

#[test]
fn without_gates_router_loop_exits_after_done_or_router_b() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let (artifacts, backups) = router_loop_artifacts(tmp.path());
        for done in [false, true] {
            let decision = decide_loop_case(
                &artifacts,
                &backups,
                LoopExitCase {
                    done,
                    request_open: false,
                    gates: false,
                    agent_loop: 1,
                    max_loops: 3,
                },
            );
            assert!(
                matches!(decision, super::RouterLoopDecision::Exit),
                "gates off must not start another iteration (done={done})"
            );
        }
    });
}

#[test]
fn gates_failure_continues_until_the_last_loop_after_done_or_router_b() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let (_bin, _guard) =
            malvin::test_support::test_agent_client::write_fake_gate(tmp.path(), "false", 1);
        let (artifacts, backups) = router_loop_artifacts(tmp.path());
        for done in [false, true] {
            let mid = decide_loop_case(
                &artifacts,
                &backups,
                LoopExitCase {
                    done,
                    request_open: false,
                    gates: true,
                    agent_loop: 1,
                    max_loops: 3,
                },
            );
            assert!(
                matches!(mid, super::RouterLoopDecision::Continue),
                "a failed gate with loops left must rerun (done={done})"
            );
            let last = decide_loop_case(
                &artifacts,
                &backups,
                LoopExitCase {
                    done,
                    request_open: false,
                    gates: true,
                    agent_loop: 3,
                    max_loops: 3,
                },
            );
            assert!(
                matches!(last, super::RouterLoopDecision::ExitGatesFailed(_)),
                "the last failed gate must stop the run (done={done})"
            );
        }
    });
}

#[test]
fn gates_pass_exits_after_done_or_router_b() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let (_bin, _guard) =
            malvin::test_support::test_agent_client::write_fake_gate(tmp.path(), "true", 0);
        let (artifacts, backups) = router_loop_artifacts(tmp.path());
        for done in [false, true] {
            let decision = decide_loop_case(
                &artifacts,
                &backups,
                LoopExitCase {
                    done,
                    request_open: false,
                    gates: true,
                    agent_loop: 1,
                    max_loops: 3,
                },
            );
            assert!(
                matches!(decision, super::RouterLoopDecision::Exit),
                "a passing gate must not start another iteration (done={done})"
            );
        }
    });
}

#[test]
fn open_request_restarts_until_the_last_loop_and_skips_gates() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let (_bin, _guard) =
            malvin::test_support::test_agent_client::write_fake_gate(tmp.path(), "false", 1);
        let (artifacts, backups) = router_loop_artifacts(tmp.path());
        let without_gates = decide_loop_case(
            &artifacts,
            &backups,
            LoopExitCase {
                done: false,
                request_open: true,
                gates: false,
                agent_loop: 1,
                max_loops: 3,
            },
        );
        assert!(
            matches!(without_gates, super::RouterLoopDecision::Continue),
            "a missing router_c marker restarts even when gates are off"
        );
        let mid = decide_loop_case(
            &artifacts,
            &backups,
            LoopExitCase {
                done: false,
                request_open: true,
                gates: true,
                agent_loop: 1,
                max_loops: 3,
            },
        );
        assert!(
            matches!(mid, super::RouterLoopDecision::Continue),
            "a missing router_c marker must start another iteration"
        );
        let last = decide_loop_case(
            &artifacts,
            &backups,
            LoopExitCase {
                done: false,
                request_open: true,
                gates: true,
                agent_loop: 3,
                max_loops: 3,
            },
        );
        assert!(
            matches!(last, super::RouterLoopDecision::Exit),
            "the last iteration must stop without treating a missing marker as a gate failure"
        );
    });
}

#[test]
fn exit_gates_failed_outranks_finalize_error() {
    use super::RouterLoopDecision;
    use super::router_flow_loop_decide::prefer_exit_gates_over_acp;
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
