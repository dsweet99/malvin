use super::*;

#[test]
fn create_run_artifacts_scaffolds_empty_quality_gates_log() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("plan", Some(tmp.path())).unwrap();
    let qlog = art.quality_gates_log_path();
    assert!(qlog.is_file(), "quality_gates.log must exist at {}", qlog.display());
    assert_eq!(std::fs::read_to_string(&qlog).unwrap(), "");
    std::fs::write(&qlog, "stale").unwrap();
    super::create::ensure_quality_gates_log_file(&art).unwrap();
    assert_eq!(std::fs::read_to_string(&qlog).unwrap(), "");
}

#[test]
fn gate_exp_log_path_is_scoped_per_iteration() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("plan", Some(tmp.path())).unwrap();
    let g1 = art.gate_exp_log_path(1);
    let g2 = art.gate_exp_log_path(2);
    assert_ne!(g1, g2);
    assert!(g1.to_string_lossy().contains("_g1.md"));
    super::create::ensure_gate_exp_log_file(&art, 1).unwrap();
    assert!(g1.is_file());
}

#[test]
fn create_run_artifacts_scaffolds_kpop_exp_log_under_run_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("plan", Some(tmp.path())).unwrap();
    let exp = art.exp_log_path();
    assert!(exp.is_file(), "exp log must exist at {}", exp.display());
    assert!(
        exp.starts_with(tmp.path().join(".malvin/logs")),
        "exp log must live under .malvin/logs, got {}",
        exp.display()
    );
    assert!(
        exp.to_string_lossy().contains("/_kpop/exp_log_"),
        "exp log must use run-scoped _kpop path, got {}",
        exp.display()
    );
}

#[test]
fn create_run_artifacts_from_plan_copy_scaffolds_kpop_exp_log() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").unwrap();
    let art = create_run_artifacts(&plan, Some(tmp.path())).unwrap();
    assert!(art.exp_log_path().is_file());
}

#[test]
fn kpop_workflow_context_exp_log_is_under_malvin_logs() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_kpop_run_artifacts("kpop body", Some(tmp.path())).unwrap();
    let exp_path = art.exp_log_path();
    assert!(exp_path.is_file());
    let ctx = crate::workflow_context::workflow_context_paths_only(&art, "kpop");
    let exp_log = ctx.get("exp_log").unwrap_or_else(|| panic!("missing exp_log: {ctx:?}"));
    let kpop_log_dir = ctx.get("kpop_log_dir").unwrap();
    assert!(
        exp_log.contains(".malvin/logs"),
        "exp_log must be under .malvin/logs, got {exp_log:?}"
    );
    assert!(
        !exp_log.starts_with("./_kpop"),
        "exp_log must not be repo-root ./_kpop, got {exp_log:?}"
    );
    assert!(
        kpop_log_dir.contains(".malvin/logs"),
        "kpop_log_dir must be under .malvin/logs, got {kpop_log_dir:?}"
    );
    assert!(
        !kpop_log_dir.starts_with("./_kpop"),
        "kpop_log_dir must not be repo-root ./_kpop, got {kpop_log_dir:?}"
    );
    assert!(
        exp_log.starts_with("./"),
        "exp_log should be relative to work_dir, got {exp_log:?}"
    );
}

#[test]
fn kpop_exp_log_path_from_repo_root_work_dir() {
    let art = create_kpop_run_artifacts_opts(
        "probe",
        Some(std::path::Path::new(".")),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .unwrap();
    let exp_path = art.exp_log_path();
    assert!(exp_path.is_file());
    let ctx = crate::workflow_context::workflow_context_paths_only(&art, "kpop");
    let exp_log = ctx.get("exp_log").cloned().unwrap_or_default();
    let kpop_log_dir = ctx.get("kpop_log_dir").cloned().unwrap_or_default();
    assert!(exp_log.contains(".malvin/logs"));
    assert!(!exp_log.starts_with("./_kpop"));
    assert!(!kpop_log_dir.starts_with("./_kpop"));
    let _ = std::fs::remove_dir_all(&art.run_dir);
}
