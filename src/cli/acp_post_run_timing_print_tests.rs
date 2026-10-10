use malvin::acp_post_run::merge_acp_restore_and_check_abort;
use malvin::output::set_stdout_log_path;
use malvin::run_timing::timing_footnote_tests::seed_run_timing_json;

#[test]
fn merge_restore_check_abort_then_print_timing_noops_without_json() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
        let work = tempfile::tempdir().unwrap();
        let empty = malvin::test_support::test_utils::empty_session_dotfile_backups(work.path());
        let artifacts =
            malvin::artifacts::create_run_artifacts_from_text("code", Some(work.path()))
                .expect("artifacts");
        merge_acp_restore_and_check_abort(Ok(()), &artifacts, &empty).expect("merge");
    });
}

#[test]
fn merge_restore_check_abort_does_not_print_timing_inside_the_request() {
    malvin::test_support::test_utils::with_isolated_home(|_| {
        let work = tempfile::tempdir().unwrap();
        let empty = malvin::test_support::test_utils::empty_session_dotfile_backups(work.path());
        let artifacts =
            malvin::artifacts::create_run_artifacts_from_text("code", Some(work.path()))
                .expect("artifacts");
        set_stdout_log_path(None);
        seed_run_timing_json(&artifacts.run_dir);

        let log_path = artifacts.run_dir.join("stdout.log");
        std::fs::write(&log_path, "").expect("truncate");
        set_stdout_log_path(Some(log_path.clone()));
        merge_acp_restore_and_check_abort(Ok(()), &artifacts, &empty).expect("merge");
        set_stdout_log_path(None);
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        assert!(
            !log.contains("TIMING:") && !log.contains("COST:"),
            "request end must not print process footnotes; log={log:?}"
        );
    });
}
