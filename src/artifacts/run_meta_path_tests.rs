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
fn trace_jsonl_path_is_under_run_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("plan", Some(tmp.path())).unwrap();
    let trace = art.run_dir.join(crate::malvin_constants::TRACE_JSONL);
    assert_eq!(trace, art.run_dir.join(crate::malvin_constants::TRACE_JSONL));
}

#[test]
fn create_run_artifacts_scaffolds_exp_log_under_run_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("plan", Some(tmp.path())).unwrap();
    let exp = art.exp_log_path();
    assert!(exp.is_file(), "exp log must exist at {}", exp.display());
    assert!(
        exp.starts_with(crate::malvin_logs_root(tmp.path())),
        "exp log must live under home malvin logs bucket, got {}",
        exp.display()
    );
    assert!(
        exp.to_string_lossy().contains("/_run/exp_log_"),
        "exp log must use run-scoped _run path, got {}",
        exp.display()
    );
}

#[test]
fn create_run_artifacts_from_plan_copy_scaffolds_exp_log() {
    let tmp = tempfile::tempdir().unwrap();
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "p").unwrap();
    let art = create_run_artifacts(&plan, Some(tmp.path())).unwrap();
    assert!(art.exp_log_path().is_file());
}

#[test]
fn router_workflow_context_exp_log_is_under_home_malvin_logs() {
    let tmp = tempfile::tempdir().unwrap();
    let art = create_run_artifacts_from_text("plan body", Some(tmp.path())).unwrap();
    let exp_path = art.exp_log_path();
    assert!(exp_path.is_file());
    let ctx = crate::workflow_context::workflow_context_paths_only(&art, crate::config::DEFAULT_CLI_MODEL, false);
    let exp_log = ctx.get("exp_log").unwrap_or_else(|| panic!("missing exp_log: {ctx:?}"));
    let run_meta_dir = ctx.get("run_meta_dir").unwrap();
    let home_logs = crate::malvin_home_logs_root();
    assert!(
        exp_log.contains(&home_logs.display().to_string())
            || exp_log.contains(".malvin_home/logs"),
        "exp_log must reference home logs tree, got {exp_log:?}"
    );
    assert!(
        run_meta_dir.contains(&home_logs.display().to_string())
            || run_meta_dir.contains(".malvin_home/logs"),
        "run_meta_dir must reference home logs tree, got {run_meta_dir:?}"
    );
    assert!(
        !exp_log.starts_with("./_run"),
        "exp_log must not be repo-root ./_run, got {exp_log:?}"
    );
    assert!(
        !run_meta_dir.starts_with("./_run"),
        "run_meta_dir must not be repo-root ./_run, got {run_meta_dir:?}"
    );
}

#[test]
fn exp_log_path_from_repo_root_work_dir() {
    let art = create_run_artifacts_from_text_opts(
        "probe",
        Some(std::path::Path::new(".")),
        crate::run_id::RunDirOptions { gc: false },
    )
    .unwrap();
    let exp_path = art.exp_log_path();
    assert!(exp_path.is_file());
    let ctx = crate::workflow_context::workflow_context_paths_only(&art, crate::config::DEFAULT_CLI_MODEL, false);
    let exp_log = ctx.get("exp_log").cloned().unwrap_or_default();
    let run_meta_dir = ctx.get("run_meta_dir").cloned().unwrap_or_default();
    assert!(
        exp_log.contains(".malvin_home/logs") || exp_log.starts_with('/'),
        "exp_log must be absolute or under .malvin_home/logs, got {exp_log:?}"
    );
    assert!(!exp_log.starts_with("./_run"));
    assert!(!run_meta_dir.starts_with("./_run"));
    let _ = std::fs::remove_dir_all(&art.run_dir);
}
