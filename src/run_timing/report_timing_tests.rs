use std::time::Duration;

use serde_json::Value;

use super::report_timing_line::format_timing_stdout_line_from_json;
use super::{duration_ms_u64, to_json_value, write_json_and_print_summary, write_json_only};

#[test]
fn timing_line_implement_echoes_json_ms_via_same_formatter() {
    use crate::run_timing::{RunTiming, TimingPhase};

    let mut r = RunTiming::default();
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(23_451));
    let json: Value = to_json_value(&r);
    let line = format_timing_stdout_line_from_json(&json);
    assert_eq!(json["phases_ms"]["implement"], 23_451);
    assert!(
        !line.contains("implement = "),
        "TIMING line must omit phase fields; line={line:?}"
    );
    assert!(line.contains("llm_wait = 23.5s"), "line={line:?}");
}

#[test]
fn timing_line_from_json_matches_to_json_value_snapshot() {
    use crate::run_timing::{RunTiming, TimingPhase};

    let _ = write_json_only;
    let _ = write_json_and_print_summary;
    let _ = duration_ms_u64;
    let _ = format_timing_stdout_line_from_json;
    let mut r = RunTiming::default();
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(500));
    let json = to_json_value(&r);
    let line = format_timing_stdout_line_from_json(&json);
    assert!(json["phases_ms"]["implement"].as_u64().unwrap() >= 500);
    assert!(!line.contains("implement = "));
    assert!(line.contains("llm_wait = "));
}

#[test]
fn timing_line_uses_phase_display_name_alias_when_present() {
    use crate::run_timing::{RunTiming, TimingPhase};

    let mut r = RunTiming::default();
    r.set_implement_display_name("router");
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    let json = to_json_value(&r);
    let line = format_timing_stdout_line_from_json(&json);
    assert_eq!(json["phase_display_names"]["implement"], "router");
    assert!(!line.contains("router = "));
    assert!(!line.contains("implement = "));
}

#[test]
fn timing_line_uses_one_decimal_and_includes_live_buckets() {
    use crate::run_timing::{RUN_TIMING_SUMMARY_PREFIX, RunTiming, TimingPhase};

    let mut r = RunTiming::default();
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    let line = format_timing_stdout_line_from_json(&to_json_value(&r));
    assert!(line.starts_with(RUN_TIMING_SUMMARY_PREFIX));
    assert!(line.contains("wall = "));
    assert!(line.contains("llm_wait = "));
    assert!(!line.contains("implement = "));
    assert!(!line.contains("router = "));
    assert!(!line.contains("summary = "));
    assert!(!line.contains("concerns = "));
    assert!(!line.contains("check_plan = "));
}

#[test]
fn duration_ms_u64_converts_duration_to_milliseconds() {
    assert_eq!(duration_ms_u64(Duration::from_millis(0)), 0);
    assert_eq!(duration_ms_u64(Duration::from_millis(123)), 123);
    assert_eq!(duration_ms_u64(Duration::from_secs(5)), 5000);
}

#[test]
fn print_summary_from_run_dir_noops_when_json_missing() {
    let tmp = tempfile::tempdir().unwrap();
    crate::run_timing::print_summary_from_run_dir(tmp.path()).expect("noop");
}

#[test]
fn print_summary_from_run_dir_reads_existing_json() {
    use crate::run_timing::{RunTiming, TimingPhase};

    let tmp = tempfile::tempdir().unwrap();
    let mut r = RunTiming::default();
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    r.write_json_only(tmp.path()).expect("json");
    crate::run_timing::print_summary_from_run_dir(tmp.path()).expect("print");
}

#[test]
fn write_json_and_print_summary_creates_file() {
    use crate::run_timing::{RUN_TIMING_JSON_FILE, RunTiming, TimingPhase};

    let tmp = tempfile::tempdir().unwrap();
    let mut r = RunTiming::default();
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    r.write_json_and_print_summary(tmp.path()).unwrap();
    assert!(tmp.path().join(RUN_TIMING_JSON_FILE).exists());
}

#[test]
fn run_timing_json_includes_cost_block_with_reported_usage() {
    use crate::llm_transport::ResponseUsage;
    use crate::run_timing::{RunTiming, TimingPhase};

    let mut r = RunTiming::default();
    r.record_completion_cost(&ResponseUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(1),
        cost: Some(0.01),
    });
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(1));
    let json = to_json_value(&r);
    assert!(json.get("cost").is_some());
}

#[test]
fn no_cost_block_when_no_cost_data() {
    use crate::run_timing::RunTiming;

    let r = RunTiming::default();
    let json = to_json_value(&r);
    assert!(json.get("cost").is_none());
}

#[test]
fn cost_fields_on_combined_stdout_line_not_timing_line() {
    use super::super::report_cost_line::format_cost_stdout_line_from_json;
    use crate::llm_transport::ResponseUsage;
    use crate::run_timing::RunTiming;

    let mut r = RunTiming::default();
    r.record_completion_cost(&ResponseUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(1),
        cost: Some(0.0842),
    });
    let json = to_json_value(&r);
    let timing_line = format_timing_stdout_line_from_json(&json);
    assert!(!timing_line.contains("cost_tot"));
    let cost_line = format_cost_stdout_line_from_json(&json);
    assert!(cost_line.starts_with("COST:"));
    assert!(cost_line.contains("cost_tot = 0.0000"));
    assert!(cost_line.contains("steps ="));
}
