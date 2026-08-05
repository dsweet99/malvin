use std::io;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::cost::cost_stats;
use super::tokens::tokens_stats;
use super::{RUN_TIMING_JSON_FILE, RunTiming};
use crate::output::{MALVIN_WHO, print_stdout_line};

#[path = "report_timing_line.rs"]
mod report_timing_line;
use report_timing_line::format_timing_stdout_line_from_json;
use super::report_cost_line::format_cost_stdout_line_from_json;

fn print_timing_and_cost_summary(json: &Value) {
    print_stdout_line(MALVIN_WHO, &format_timing_stdout_line_from_json(json));
    print_stdout_line(MALVIN_WHO, &format_cost_stdout_line_from_json(json));
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
        "tool_calls_by_type_ms": {
            "read": ms(r.tool_calls_read), "search": ms(r.tool_calls_search),
            "edit": ms(r.tool_calls_edit), "execute": ms(r.tool_calls_execute),
            "other": ms(r.tool_calls_other),
        },
        "phases_ms": { "implement": ms(r.implement) },
        "tokens": tokens_stats(r),
    });
    if let Some(cost) = cost_stats(r) {
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


#[cfg(test)]
#[path = "report_timing_tests.rs"]
mod report_timing_tests;

#[cfg(test)]
#[path = "report_tokens_tests.rs"]
mod report_tokens_tests;
