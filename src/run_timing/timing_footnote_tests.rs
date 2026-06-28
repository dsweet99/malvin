use super::{RunTiming, finalize_and_emit_run_timing, print_summary_from_run_dir};
use std::time::Instant;

#[test]
fn success_footnotes_emit_timing_before_done_and_done_is_last() {
    let _locks = stdout_and_phase_test_locks();
    let run_dir = tempfile::tempdir().expect("run_dir");
    seed_run_timing_json(run_dir.path());
    let log = capture_timing_then_done_log(run_dir.path());
    assert_timing_precedes_done_and_done_is_last(&log);
}

fn stdout_and_phase_test_locks() -> (
    std::sync::MutexGuard<'static, ()>,
    std::sync::MutexGuard<'static, ()>,
) {
    let stdout = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let phase = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    (stdout, phase)
}

fn seed_run_timing_json(run_dir: &std::path::Path) {
    use malvin_mini::ResponseUsage;

    let timing = RunTiming::new_arc();
    {
        let mut g = timing.lock().unwrap();
        g.mark_wall_start(Instant::now());
        g.mark_wall_end(Instant::now());
        g.record_mini_http_cost(&ResponseUsage {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: Some(1),
            cost: Some(0.0842),
        });
    }
    finalize_and_emit_run_timing(run_dir, &timing).expect("emit");
}

fn capture_timing_then_done_log(run_dir: &std::path::Path) -> String {
    let log_path = run_dir.join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    print_summary_from_run_dir(run_dir).expect("timing");
    crate::agent_phase::print_done_with_reporting_phase();
    crate::output::set_stdout_log_path(None);
    std::fs::read_to_string(log_path).unwrap_or_default()
}

fn assert_timing_precedes_done_and_done_is_last(log: &str) {
    let timing_pos = log.find("TIMING:").expect("TIMING line");
    let done_pos = log.find("DONE").expect("DONE line");
    assert!(
        timing_pos < done_pos,
        "TIMING must precede DONE; log={log:?}"
    );
    if let Some(cost_pos) = log.find("COST:") {
        assert!(
            timing_pos < cost_pos && cost_pos < done_pos,
            "COST must follow TIMING and precede DONE; log={log:?}"
        );
    }
    let malvin_lines: Vec<&str> = log
        .lines()
        .filter(|line: &&str| line.contains(" o|"))
        .collect();
    let last = malvin_lines.last().copied().unwrap_or("");
    assert!(
        last.contains("DONE"),
        "DONE must be the last malvin stdout line; last={last:?} log={log:?}"
    );
}
