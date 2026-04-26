//! JSON (`run_timing.json`) plus one **stdout** summary line for [`super::RunTiming`].
//! The printed line uses [`super::RUN_TIMING_SUMMARY_PREFIX`] (`TIMING: `, colon plus trailing ASCII space) before the first `name = value` field—see [`format_timing_stdout_line_from_json`].

use std::io;
use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};

use super::{RUN_TIMING_JSON_FILE, RUN_TIMING_SUMMARY_PREFIX, RunTiming};
use crate::output::{MALVIN_WHO, print_stdout_line};

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
        "phase_display_names": {
            "implement": r.implement_display_name,
        },
        "phases_ms": {
            "check_plan": ms(r.check_plan),
            "implement": ms(r.implement),
            "review_1_review": ms(r.review_1_review),
            "review_2_review": ms(r.review_2_review),
            "concerns": ms(r.concerns),
            "learn": ms(r.learn),
        }
    })
}

pub(super) fn write_json_only(r: &RunTiming, run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    let file = std::fs::File::create(&path)?;
    let json = to_json_value(r);
    serde_json::to_writer_pretty(file, &json)?;
    Ok(())
}

/// Phase keys under `phases_ms` in [`to_json_value`] — keep order aligned with [`format_timing_stdout_line_from_json`].
const PHASE_MS_KEYS_JSON_ORDER: [&str; 6] = [
    "check_plan",
    "implement",
    "review_1_review",
    "review_2_review",
    "concerns",
    "learn",
];

/// Writes `run_timing.json` and prints one stdout summary line (timestamp-prefixed).
pub(super) fn write_json_and_print_summary(r: &RunTiming, run_dir: &Path) -> io::Result<()> {
    let path = run_dir.join(RUN_TIMING_JSON_FILE);
    let file = std::fs::File::create(&path)?;
    let json = to_json_value(r);
    serde_json::to_writer_pretty(file, &json)?;

    print_stdout_line(MALVIN_WHO, &format_timing_stdout_line_from_json(&json));
    Ok(())
}

fn format_ms_one_decimal_s(ms: u64) -> String {
    let tenth_secs = (ms.saturating_add(50)) / 100;
    let whole = tenth_secs / 10;
    let frac = tenth_secs % 10;
    format!("{whole}.{frac}s")
}

fn timing_line_append_part(out: &mut String, first: &mut bool, key: &str, val: &str) {
    use std::fmt::Write;
    if !*first {
        out.push(' ');
    }
    *first = false;
    let _ = write!(out, "{key} = {val}");
}

fn phase_display_name<'a>(v: &'a Value, key: &'a str) -> &'a str {
    v.get("phase_display_names")
        .and_then(Value::as_object)
        .and_then(|o| o.get(key))
        .and_then(Value::as_str)
        .unwrap_or(key)
}

/// Builds the stdout timing summary line (prefix [`RUN_TIMING_SUMMARY_PREFIX`], i.e. `TIMING: ` including the trailing space before the first `name = value` field) from the same [`serde_json::Value`] written to `run_timing.json`, so fields stay aligned.
fn format_timing_stdout_line_from_json(v: &Value) -> String {
    let mut s = String::from(RUN_TIMING_SUMMARY_PREFIX);
    let mut first = true;
    match v.get("wall_clock_ms").and_then(Value::as_u64) {
        Some(ms) => {
            timing_line_append_part(&mut s, &mut first, "wall", &format_ms_one_decimal_s(ms));
        }
        None => timing_line_append_part(&mut s, &mut first, "wall", "n/a"),
    }
    for (json_key, line_key) in [
        ("llm_wait_ms", "llm_wait"),
        ("agent_retry_backoff_ms", "agent_retry_backoff"),
    ] {
        let ms = v.get(json_key).and_then(Value::as_u64).unwrap_or(0);
        timing_line_append_part(&mut s, &mut first, line_key, &format_ms_one_decimal_s(ms));
    }
    let phases = v.get("phases_ms").and_then(Value::as_object);
    for key in PHASE_MS_KEYS_JSON_ORDER {
        let ms = phases
            .and_then(|o| o.get(key))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        timing_line_append_part(
            &mut s,
            &mut first,
            phase_display_name(v, key),
            &format_ms_one_decimal_s(ms),
        );
    }
    s
}

#[cfg(test)]
mod format_tests {
    use std::time::Duration;

    use serde_json::Value;

    use super::format_ms_one_decimal_s;

    #[test]
    fn format_ms_one_decimal_s_rounds_to_tenths() {
        assert_eq!(format_ms_one_decimal_s(100), "0.1s");
        assert_eq!(format_ms_one_decimal_s(23451), "23.5s");
    }

    #[test]
    fn timing_line_implement_echoes_json_ms_via_same_formatter() {
        use crate::run_timing::{RunTiming, TimingPhase};

        let mut r = RunTiming::default();
        r.mark_wall_start(std::time::Instant::now());
        r.mark_wall_end(std::time::Instant::now());
        r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(23_451));
        let json: Value = super::to_json_value(&r);
        let implement_ms = json["phases_ms"]["implement"]
            .as_u64()
            .expect("implement ms in json");
        let expected = format_ms_one_decimal_s(implement_ms);
        let line = super::format_timing_stdout_line_from_json(&json);
        assert!(
            line.contains(&format!("implement = {expected}")),
            "TIMING line should use format_ms_one_decimal_s(phases_ms.implement); line={line:?} json={json}"
        );
    }

    #[test]
    fn timing_line_from_json_matches_to_json_value_snapshot() {
        use crate::run_timing::{RunTiming, TimingPhase};

        let mut r = RunTiming::default();
        r.mark_wall_start(std::time::Instant::now());
        r.mark_wall_end(std::time::Instant::now());
        r.add_llm_phase(TimingPhase::Concerns, Duration::from_millis(500));
        let json = super::to_json_value(&r);
        let line = super::format_timing_stdout_line_from_json(&json);
        assert!(line.contains("concerns = "));
    }

    #[test]
    fn timing_line_uses_phase_display_name_alias_when_present() {
        use crate::run_timing::{RunTiming, TimingPhase};

        let mut r = RunTiming::default();
        r.set_implement_display_name("raw");
        r.mark_wall_start(std::time::Instant::now());
        r.mark_wall_end(std::time::Instant::now());
        r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
        let json = super::to_json_value(&r);
        let line = super::format_timing_stdout_line_from_json(&json);
        assert_eq!(json["phase_display_names"]["implement"], "raw");
        assert!(line.contains("raw = "));
        assert!(!line.contains("implement = "));
    }

    #[test]
    fn timing_line_uses_one_decimal_and_includes_all_buckets() {
        use crate::run_timing::{RUN_TIMING_SUMMARY_PREFIX, RunTiming, TimingPhase};

        let mut r = RunTiming::default();
        r.mark_wall_start(std::time::Instant::now());
        r.mark_wall_end(std::time::Instant::now());
        r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
        let line = super::format_timing_stdout_line_from_json(&super::to_json_value(&r));
        assert!(line.starts_with(RUN_TIMING_SUMMARY_PREFIX));
        assert!(line.contains("wall = "));
        assert!(line.contains("llm_wait = "));
        assert!(line.contains("implement = "));
        assert!(line.contains("learn = "));
        assert!(line.contains(&format_ms_one_decimal_s(100)));
    }

    #[test]
    fn duration_ms_u64_converts_duration_to_milliseconds() {
        assert_eq!(super::duration_ms_u64(Duration::from_millis(0)), 0);
        assert_eq!(super::duration_ms_u64(Duration::from_millis(123)), 123);
        assert_eq!(super::duration_ms_u64(Duration::from_secs(5)), 5000);
    }

    #[test]
    fn timing_line_append_part_formats_key_value_pairs() {
        let mut out = String::new();
        let mut first = true;
        super::timing_line_append_part(&mut out, &mut first, "foo", "1.0s");
        assert_eq!(out, "foo = 1.0s");
        assert!(!first);
        super::timing_line_append_part(&mut out, &mut first, "bar", "2.0s");
        assert_eq!(out, "foo = 1.0s bar = 2.0s");
    }

    #[test]
    fn phase_display_name_returns_alias_or_key() {
        let json: Value = serde_json::json!({
            "phase_display_names": { "implement": "raw" }
        });
        assert_eq!(super::phase_display_name(&json, "implement"), "raw");
        assert_eq!(super::phase_display_name(&json, "learn"), "learn");
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
}
