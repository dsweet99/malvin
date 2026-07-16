use super::{maybe_run_router_post_c_gates, RouterPostCGates, RouterTurnsOutcome};
use crate::artifacts::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigBackup, MalvinConfigWorkspaceBackup,
    SessionDotfileBackups, VisionBackup,
};

#[test]
fn router_turns_outcome_exposes_gate_continue_flag() {
    let outcome = RouterTurnsOutcome {
        iteration_backups: SessionDotfileBackups::from_parts(
            crate::session_dotfile_backup::SessionDotfileParts {
                malvin_checks: MalvinChecksBackup::Missing,
                malvin_config: MalvinConfigBackup::Missing,
                gitignore: GitignoreBackup::Missing,
                vision: VisionBackup::Missing,
                malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
            },
        ),
        gate_wants_continue: true,
    };
    assert!(outcome.gate_wants_continue);
}

#[test]
fn maybe_run_router_post_c_gates_skips_when_not_coding_task() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    assert!(!maybe_run_router_post_c_gates(
        tmp.path(),
        &run_dir,
        RouterPostCGates {
            coding_task: false,
            enabled: true,
        },
    ));
}

#[test]
fn maybe_run_router_post_c_gates_skips_when_gates_are_off() {
    let tmp = tempfile::tempdir().expect("tempdir");
    crate::seed_malvin_checks(tmp.path(), "false\n");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    assert!(!maybe_run_router_post_c_gates(
        tmp.path(),
        &run_dir,
        RouterPostCGates {
            coding_task: true,
            enabled: false,
        },
    ));
    assert!(!run_dir.join("quality_gates.log").exists());
}

#[test]
fn maybe_run_router_post_c_gates_passes_when_checks_succeed() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    assert!(!maybe_run_router_post_c_gates(
        tmp.path(),
        &run_dir,
        RouterPostCGates {
            coding_task: true,
            enabled: true,
        },
    ));
}

#[test]
fn maybe_run_router_post_c_gates_wants_continue_when_check_fails() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    crate::seed_malvin_checks(tmp.path(), "false\n");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("run dir");
    assert!(maybe_run_router_post_c_gates(
        tmp.path(),
        &run_dir,
        RouterPostCGates {
            coding_task: true,
            enabled: true,
        },
    ));
    let qlog = std::fs::read_to_string(run_dir.join("quality_gates.log")).expect("quality_gates.log");
    assert!(qlog.contains("false"));
}
