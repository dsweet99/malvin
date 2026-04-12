//! JSON + stdout summary for [`super::RunTiming`].

use std::io;
use std::path::Path;
use std::time::Duration;

use chrono::Local;
use serde_json::{Value, json};

use super::{RunTiming, RUN_TIMING_JSON_FILE, RUN_TIMING_SUMMARY_PREFIX};

pub(super) fn duration_ms_u64(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

pub(super) fn to_json_value(r: &RunTiming) -> Value {
    let wall_ms = r.wall_duration().map(duration_ms_u64);
    let ms = duration_ms_u64;
    json!({
        "wall_clock_ms": wall_ms,
        "llm_wait_ms": ms(r.llm_wait),
        "agent_retry_backoff_ms": ms(r.agent_retry_backoff),
        "phases_ms": {
            "implement": ms(r.implement),
            "review_1_review": ms(r.review_1_review),
            "review_1_kpop": ms(r.review_1_kpop),
            "review_2_review": ms(r.review_2_review),
            "review_2_kpop": ms(r.review_2_kpop),
            "concerns": ms(r.concerns),
            "learn": ms(r.learn),
        }
    })
}

/// Seconds with three fractional digits, using the same truncated-millisecond quantization as JSON.
fn format_duration_secs_3_from_ms(ms: u64) -> String {
    let whole = ms / 1000;
    let frac = ms % 1000;
    format!("{whole}.{frac:03}s")
}

/// Writes `run_timing.json` and prints one stdout summary line (timestamp-prefixed).
pub(super) fn write_json_and_print_summary(r: &RunTiming, run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    let file = std::fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, &to_json_value(r))?;

    let now = Local::now();
    let ts = format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    );
    let wall = r.wall_duration().map_or_else(
        || "n/a".to_string(),
        |d| format_duration_secs_3_from_ms(duration_ms_u64(d)),
    );
    let llm = format_duration_secs_3_from_ms(duration_ms_u64(r.llm_wait));
    let backoff = format_duration_secs_3_from_ms(duration_ms_u64(r.agent_retry_backoff));
    println!("{ts} {RUN_TIMING_SUMMARY_PREFIX} wall {wall}; LLM wait {llm}; agent retry/backoff {backoff} (see {RUN_TIMING_JSON_FILE})");
    Ok(())
}

#[cfg(test)]
mod format_tests {
    use std::time::Duration;

    use super::{duration_ms_u64, format_duration_secs_3_from_ms};

    #[test]
    fn duration_formats_as_seconds_with_three_fractional_digits() {
        assert_eq!(
            format_duration_secs_3_from_ms(duration_ms_u64(Duration::from_millis(23_451))),
            "23.451s"
        );
    }

    #[test]
    fn stdout_seconds_match_json_truncated_milliseconds() {
        let d = Duration::from_millis(1500) + Duration::from_micros(500);
        let ms = duration_ms_u64(d);
        let s = format_duration_secs_3_from_ms(ms);
        let (whole, frac) = s.strip_suffix('s').unwrap().split_once('.').unwrap();
        assert_eq!(frac.len(), 3);
        let ms_round_trip: u64 = whole.parse::<u64>().unwrap() * 1000 + frac.parse::<u64>().unwrap();
        assert_eq!(
            ms, ms_round_trip,
            "summary seconds must encode the same truncated-ms value as JSON"
        );
    }
}
