use crate::cli::bug_flow::{kpop_args_from_bug, validate_bug_cli};
use crate::cli::bug_flow_remediation::{
    artifacts_for_fix_by_id, gate_retry_command, BUG_KPOP_REQUEST,
};
use crate::cli::BugArgs;

#[test]
fn kpop_args_from_bug_maps_bug_fields() {
    let bug = BugArgs {
        max_hypotheses: 7,
        no_learn: true,
        skip_pre_checks: true,
        fix: false,
        bug_id: None,
    };
    let kpop = kpop_args_from_bug(&bug);
    assert_eq!(kpop.max_hypotheses, 7);
    assert!(kpop.no_learn);
    assert_eq!(kpop.request.as_deref(), Some(BUG_KPOP_REQUEST));
}

#[test]
fn gate_retry_command_fix_by_id() {
    let bug = BugArgs {
        max_hypotheses: 10,
        no_learn: false,
        skip_pre_checks: false,
        fix: false,
        bug_id: Some("Ma1b2c".to_string()),
    };
    assert_eq!(gate_retry_command(&bug), "malvin hunt Ma1b2c");
}

#[test]
fn gate_retry_command_discover_fix() {
    let bug = BugArgs {
        max_hypotheses: 10,
        no_learn: false,
        skip_pre_checks: false,
        fix: true,
        bug_id: None,
    };
    assert_eq!(gate_retry_command(&bug), "malvin hunt --fix");
}

#[test]
fn validate_bug_cli_rejects_fix_with_id() {
    let bug = BugArgs {
        max_hypotheses: 1,
        no_learn: true,
        skip_pre_checks: true,
        fix: true,
        bug_id: Some("Ma1b2c".to_string()),
    };
    let err = validate_bug_cli(&bug).unwrap_err();
    assert!(err.contains("--fix"));
}

#[test]
fn artifacts_for_fix_by_id_writes_plan_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let resolved = crate::cli::bug_id_lookup::BugIdResolved {
        run_dir: run_dir.clone(),
        exp_log_path: run_dir.join("_kpop").join("exp_log_run.md"),
        work_dir: tmp.path().to_path_buf(),
    };
    let artifacts = artifacts_for_fix_by_id(&resolved).expect("artifacts");
    assert!(artifacts.plan_path.is_file());
    let text = std::fs::read_to_string(artifacts.plan_path).expect("read plan");
    assert!(text.contains("KPOP_SOLVED"));
}
