//! JSON + stderr summary for [`super::RunTiming`].

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

fn format_hms(d: Duration) -> String {
    let secs = d.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{h}h{m:02}m{s:02}s")
    } else if m > 0 {
        format!("{m}m{s:02}s")
    } else {
        format!("{s}s")
    }
}

/// Writes `run_timing.json` and prints one stderr summary line (timestamp-prefixed).
pub(super) fn write_json_and_eprint_summary(r: &RunTiming, run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    let file = std::fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, &to_json_value(r))?;

    let now = Local::now();
    let ts = format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    );
    let wall = r
        .wall_duration()
        .map_or_else(|| "n/a".to_string(), format_hms);
    let llm = format_hms(r.llm_wait);
    let backoff = format_hms(r.agent_retry_backoff);
    eprintln!("{ts} {RUN_TIMING_SUMMARY_PREFIX} wall {wall}; LLM wait {llm}; agent retry/backoff {backoff} (see {RUN_TIMING_JSON_FILE})");
    Ok(())
}
