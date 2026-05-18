// Artifact and run-id smoke tests.


use crate::artifacts::{
    KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
};

fn exercise_empty_dotfile_backup_slots(tmp: &Path) {
    let miss = KissConfigBackup::Missing;
    assert!(crate::artifacts::backup_workspace_kissconfig_if_present(tmp).is_ok());
    assert!(crate::artifacts::backup_workspace_malvin_checks_if_present(tmp).is_ok());
    assert!(crate::artifacts::backup_workspace_kissignore_if_present(tmp).is_ok());
    crate::artifacts::restore_workspace_kissconfig_backup(tmp, &miss).expect("kissconfig rs");
    crate::artifacts::restore_workspace_malvin_checks_backup(tmp, &miss).expect("checks rs");
    crate::artifacts::restore_workspace_kissignore_backup(tmp, &miss).expect("ignore rs");
}

#[test]
fn smoke_create_run_artifacts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "x").expect("write plan");
    let art = crate::artifacts::create_run_artifacts(&plan, Some(tmp.path())).expect("artifacts");
    assert!(art.run_dir.exists());
    assert!(art.log_path("x").ends_with("x.log"));
    assert!(art.artifact_review_md().ends_with("review.md"));
    assert!(art.artifact_result_md().ends_with("result.md"));
    assert!(art.workspace_review_md().ends_with("review.md"));
    assert_eq!(art.exp_log_path().extension().and_then(|s| s.to_str()), Some("md"));
    assert_eq!(
        art.quality_gates_log_path().file_name().unwrap(),
        crate::artifacts::QUALITY_GATES_LOG
    );
}

#[test]
fn smoke_artifacts_kpop_and_run_id() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let art =
        crate::artifacts::create_kpop_run_artifacts("req", Some(tmp.path())).expect("kpop art");
    assert!(art.plan_path.ends_with("request.md"));
    let _ = crate::run_id::build_identifier();
    let id = crate::run_id::random_alnum(4);
    assert_eq!(id.len(), 4);
    let run_dir = crate::run_id::create_run_dir(Some(tmp.path())).expect("create_run_dir");
    assert!(run_dir.is_dir());
}

#[test]
fn smoke_artifacts_resolve_and_paths() {
    let (lit, wd) = crate::artifacts::resolve_user_request("hello").expect("literal");
    assert_eq!(lit, "hello");
    assert_eq!(wd, Path::new("."));

    let tmp = tempfile::tempdir().expect("tempdir");
    let f = tmp.path().join("doc.md");
    std::fs::write(&f, "at").expect("write");
    let rest = f.to_string_lossy();
    let at_arg = format!("@{rest}");
    let (text, _) = crate::artifacts::resolve_user_request(&at_arg).expect("at-file");
    assert_eq!(text, "at");

    assert_eq!(
        crate::artifacts::work_dir_for_path(&f),
        tmp.path().to_path_buf()
    );

    assert_eq!(
        crate::artifacts::startup_request_tag_label("@/tmp/x/plan.md"),
        "plan"
    );
}

#[test]
fn smoke_artifacts_dotfile_roundtrip() {
    let tmp = tempfile::tempdir().expect("tempdir");
    exercise_empty_dotfile_backup_slots(tmp.path());
    let bundle = SessionDotfileBackups::from_parts(
        KissConfigBackup::Missing,
        MalvinChecksBackup::Missing,
        KissignoreBackup::Missing,
    );
    crate::artifacts::restore_workspace_session_dotfiles(tmp.path(), &bundle).expect("restore all");

    let snap = SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
    snap.restore(tmp.path()).expect("snap restore");
}
