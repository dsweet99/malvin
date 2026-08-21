use crate::acp_post_run::merge_acp_restore_check_abort_then_print_timing;
use crate::output::set_stdout_log_path;
use crate::run_timing::timing_footnote_tests::seed_run_timing_json;

macro_rules! assert_timing_and_cost {
    ($log:expr) => {{
        let log = $log;
        assert!(
            log.contains("TIMING:") && log.contains("COST: steps =") && !log.contains("TOKENS:"),
            "timing/cost footnotes must have the required form: {log:?}"
        );
    }};
}

#[test]
fn merge_restore_check_abort_then_print_timing_noops_without_json() {
    crate::test_utils::with_isolated_home(|_| {
        let work = tempfile::tempdir().unwrap();
        let empty = crate::test_utils::empty_session_dotfile_backups(work.path());
        let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(work.path()))
            .expect("artifacts");
        merge_acp_restore_check_abort_then_print_timing(Ok(()), &artifacts, &empty).expect("merge");
    });
}

#[test]
fn merge_restore_check_abort_then_print_timing_emits_timing_and_cost() {
    crate::test_utils::with_isolated_home(|_| {
        let work = tempfile::tempdir().unwrap();
        let empty = crate::test_utils::empty_session_dotfile_backups(work.path());
        let artifacts = crate::artifacts::create_run_artifacts_from_text("code", Some(work.path()))
            .expect("artifacts");
        seed_run_timing_json(&artifacts.run_dir);

        let log_path = artifacts.run_dir.join("stdout.log");
        set_stdout_log_path(Some(log_path.clone()));
        merge_acp_restore_check_abort_then_print_timing(Ok(()), &artifacts, &empty).expect("merge");
        let log = std::fs::read_to_string(&log_path).unwrap_or_default();
        assert_timing_and_cost!(&log);
    });
}
