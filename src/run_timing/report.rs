use std::io;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::cost::cost_stats;
use super::{RUN_TIMING_JSON_FILE, RunTiming};
use crate::output::{MALVIN_WHO, print_stdout_line};

#[path = "report_timing_line.rs"]
mod report_timing_line;
use report_timing_line::format_timing_stdout_line_from_json;
use super::report_cost_line::format_cost_stdout_line_from_json;

fn print_timing_and_cost_summary(json: &Value) {
    print_stdout_line(MALVIN_WHO, &format_timing_stdout_line_from_json(json));
    if let Some(cost_line) = format_cost_stdout_line_from_json(json) {
        print_stdout_line(MALVIN_WHO, &cost_line);
    }
}

pub(super) fn duration_ms_u64(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

pub(super) fn wall_clock_ms_for_json(r: &RunTiming) -> Option<u64> {
    r.wall_duration()
        .map(duration_ms_u64)
        .or_else(|| {
            r.wall_start
                .map(|_| duration_ms_u64(r.elapsed_so_far()))
        })
}

pub(super) fn to_json_value(r: &RunTiming) -> Value {
    let wall_ms = wall_clock_ms_for_json(r);
    let ms = duration_ms_u64;
    let mut obj = json!({
        "wall_clock_ms": wall_ms,
        "llm_wait_ms": ms(r.llm_wait),
        "agent_retry_backoff_ms": ms(r.agent_retry_backoff),
        "phase_display_names": {
            "implement": r.implement_display_name,
        },
        "tool_calls_ms": ms(r.tool_calls),
        "phases_ms": {
            "implement": ms(r.implement),
        }
    });
    if let Some(cost) = cost_stats(&r.tx_costs, r.unknown_tx_count) {
        if let Some(map) = obj.as_object_mut() {
            map.insert("cost".into(), cost);
        }
    }
    obj
}

pub(super) fn write_json_only(r: &RunTiming, run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    let file = std::fs::File::create(&path)?;
    let json = to_json_value(r);
    serde_json::to_writer_pretty(file, &json)?;
    Ok(())
}

/// Writes `run_timing.json` and prints tagged stdout summary line(s).
pub(super) fn write_json_and_print_summary(r: &RunTiming, run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    let file = std::fs::File::create(&path)?;
    let json = to_json_value(r);
    serde_json::to_writer_pretty(file, &json)?;

    print_timing_and_cost_summary(&json);
    Ok(())
}

/// Prints the tagged stdout summary from an existing `run_timing.json`, if present.
///
/// # Errors
///
/// Returns [`std::io::Error`] when reading under `run_dir` fails.
pub fn print_summary_from_run_dir(run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    if !path.is_file() {
        return Ok(());
    }
    let file = std::fs::File::open(path)?;
    let json: Value = serde_json::from_reader(file)?;
    print_timing_and_cost_summary(&json);
    Ok(())
}

#[test]
fn timing_line_implement_echoes_json_ms_via_same_formatter() {
    use crate::run_timing::{RunTiming, TimingPhase};

    let mut r = RunTiming::default();
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(23_451));
    let json: Value = to_json_value(&r);
    let line = format_timing_stdout_line_from_json(&json);
    assert!(
        line.contains("implement = 23.5s"),
        "TIMING line should round implement ms; line={line:?} json={json}"
    );
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
    assert!(line.contains("implement = "));
}

#[test]
fn timing_line_uses_phase_display_name_alias_when_present() {
    use crate::run_timing::{RunTiming, TimingPhase};

    let mut r = RunTiming::default();
    r.set_implement_display_name("raw");
    r.mark_wall_start(std::time::Instant::now());
    r.mark_wall_end(std::time::Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    let json = to_json_value(&r);
    let line = format_timing_stdout_line_from_json(&json);
    assert_eq!(json["phase_display_names"]["implement"], "raw");
    assert!(line.contains("raw = "));
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
    assert!(line.contains("implement = 0.1s"));
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
    super::print_summary_from_run_dir(tmp.path()).expect("noop");
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
    super::print_summary_from_run_dir(tmp.path()).expect("print");
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
fn run_timing_json_includes_cost_block_under_mini() {
    use crate::run_timing::{RunTiming, TimingPhase};
    use malvin_mini::ResponseUsage;

    let mut r = RunTiming::default();
    r.record_mini_http_cost(&ResponseUsage {
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
fn no_cost_line_when_no_cost_data() {
    use super::report_cost_line::format_cost_stdout_line_from_json;
    use crate::run_timing::RunTiming;

    let r = RunTiming::default();
    let json = to_json_value(&r);
    assert!(format_cost_stdout_line_from_json(&json).is_none());
}

#[test]
fn cost_fields_on_separate_stdout_line_not_timing_line() {
    use super::report_cost_line::format_cost_stdout_line_from_json;
    use crate::run_timing::RunTiming;
    use malvin_mini::ResponseUsage;

    let mut r = RunTiming::default();
    r.record_mini_http_cost(&ResponseUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(1),
        cost: Some(0.0842),
    });
    let json = to_json_value(&r);
    let timing_line = format_timing_stdout_line_from_json(&json);
    assert!(!timing_line.contains("total_cost"));
    let cost_line = format_cost_stdout_line_from_json(&json).expect("cost line");
    assert!(cost_line.starts_with("COST:"));
    assert!(cost_line.contains("total_cost = 0.0842"));
}
