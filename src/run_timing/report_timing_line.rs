use serde_json::Value;

use crate::run_timing::{RUN_TIMING_SUMMARY_PREFIX, TOOL_CALL_TYPE_MS_KEYS};

const PHASE_MS_KEYS_JSON_ORDER: [&str; 1] = ["implement"];

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

fn timing_stdout_append_tool_calls_by_type(s: &mut String, first: &mut bool, v: &Value) {
    let by_type = v.get("tool_calls_by_type_ms").and_then(Value::as_object);
    for key in TOOL_CALL_TYPE_MS_KEYS {
        let ms = by_type
            .and_then(|o| o.get(key))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        if ms > 0 {
            timing_line_append_part(s, first, key, &format_ms_one_decimal_s(ms));
        }
    }
}

fn timing_stdout_append_fixed_ms_fields(s: &mut String, first: &mut bool, v: &Value) {
    match v.get("wall_clock_ms").and_then(Value::as_u64) {
        Some(ms) => timing_line_append_part(s, first, "wall", &format_ms_one_decimal_s(ms)),
        None => timing_line_append_part(s, first, "wall", "n/a"),
    }
    let llm_wait_ms = v.get("llm_wait_ms").and_then(Value::as_u64).unwrap_or(0);
    timing_line_append_part(s, first, "llm_wait", &format_ms_one_decimal_s(llm_wait_ms));
    if let Some(ms) = v.get("tool_calls_ms").and_then(Value::as_u64) {
        timing_line_append_part(s, first, "tool_calls", &format_ms_one_decimal_s(ms));
    }
    timing_stdout_append_tool_calls_by_type(s, first, v);
}
fn timing_stdout_append_phase_fields(s: &mut String, first: &mut bool, v: &Value) {
    let phases = v.get("phases_ms").and_then(Value::as_object);
    for key in PHASE_MS_KEYS_JSON_ORDER {
        let ms = phases
            .and_then(|o| o.get(key))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        timing_line_append_part(
            s,
            first,
            phase_display_name(v, key),
            &format_ms_one_decimal_s(ms),
        );
    }
}

pub(super) fn format_timing_stdout_line_from_json(v: &Value) -> String {
    let mut s = String::from(RUN_TIMING_SUMMARY_PREFIX);
    let mut first = true;
    timing_stdout_append_fixed_ms_fields(&mut s, &mut first, v);
    timing_stdout_append_phase_fields(&mut s, &mut first, v);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn timing_line_omits_cost_fields_when_present_in_json() {
        let json = json!({
            "wall_clock_ms": 1000,
            "llm_wait_ms": 100,
            "phases_ms": { "implement": 100 },
            "cost": {
                "cost_in": 0.05,
                "cost_out": 0.03,
                "cost_read": 0.002,
                "cost_write": 0.0022,
                "cost_tot": 0.0842
            }
        });
        let line = format_timing_stdout_line_from_json(&json);
        assert!(!line.contains("cost_tot"));
        assert!(!line.contains("cost_in"));
    }

    #[test]
    fn format_ms_one_decimal_s_rounds_to_tenths() {
        assert_eq!(format_ms_one_decimal_s(100), "0.1s");
        assert_eq!(format_ms_one_decimal_s(23451), "23.5s");
    }

    #[test]
    fn timing_line_wall_na_when_missing_from_json() {
        let json = json!({ "phases_ms": {} });
        let line = format_timing_stdout_line_from_json(&json);
        assert!(line.contains("wall = n/a"));
    }

    #[test]
    fn timing_stdout_append_helpers_cover_branches() {
        let _ = timing_stdout_append_fixed_ms_fields;
        let _ = timing_stdout_append_phase_fields;
        let mut s = String::new();
        let mut first = true;
        timing_stdout_append_fixed_ms_fields(
            &mut s,
            &mut first,
            &json!({
                "wall_clock_ms": 500,
                "llm_wait_ms": 10,
                "agent_retry_backoff_ms": 20,
                "tool_calls_ms": 30
            }),
        );
        timing_stdout_append_phase_fields(
            &mut s,
            &mut first,
            &json!({ "phases_ms": { "implement": 40 } }),
        );
        assert!(s.contains("wall = "));
        assert!(s.contains("tool_calls = "));
        assert!(s.contains("implement = "));
        assert!(!s.contains("agent_retry_backoff = "));
    }

    #[test]
    fn timing_line_append_part_formats_key_value_pairs() {
        let mut out = String::new();
        let mut first = true;
        timing_line_append_part(&mut out, &mut first, "foo", "1.0s");
        assert_eq!(out, "foo = 1.0s");
        assert!(!first);
        timing_line_append_part(&mut out, &mut first, "bar", "2.0s");
        assert_eq!(out, "foo = 1.0s bar = 2.0s");
    }

    #[test]
    fn timing_line_includes_tool_calls_and_implement_bucket() {
        let json = json!({
            "wall_clock_ms": 1000,
            "llm_wait_ms": 100,
            "agent_retry_backoff_ms": 50,
            "tool_calls_ms": 200,
            "phases_ms": {
                "implement": 2
            }
        });
        let line = format_timing_stdout_line_from_json(&json);
        assert!(line.contains("tool_calls = "));
        assert!(line.contains("implement = "));
        assert!(!line.contains("agent_retry_backoff = "));
        assert!(!line.contains("summary = "));
        assert!(!line.contains("concerns = "));
    }

    #[test]
    fn timing_line_includes_nonzero_tool_calls_by_type_after_aggregate() {
        let json = json!({
            "wall_clock_ms": 1000,
            "llm_wait_ms": 100,
            "tool_calls_ms": 350,
            "tool_calls_by_type_ms": {
                "read": 100,
                "search": 0,
                "edit": 50,
                "execute": 200,
                "other": 0
            },
            "phases_ms": { "implement": 100 }
        });
        let line = format_timing_stdout_line_from_json(&json);
        assert!(line.contains("tool_calls = 0.4s"));
        let tool_pos = line.find("tool_calls = ").expect("aggregate");
        let read_pos = line.find("read = ").expect("read");
        let edit_pos = line.find("edit = ").expect("edit");
        let execute_pos = line.find("execute = ").expect("execute");
        assert!(tool_pos < read_pos && read_pos < edit_pos && edit_pos < execute_pos);
        assert!(!line.contains("search = "));
        assert!(!line.contains("other = "));
        assert!(line.contains("read = 0.1s"));
        assert!(line.contains("edit = 0.1s"));
        assert!(line.contains("execute = 0.2s"));
    }

    #[test]
    fn timing_line_omits_agent_retry_backoff_even_when_present_in_json() {
        let json = json!({
            "wall_clock_ms": 532_200,
            "llm_wait_ms": 505_400,
            "agent_retry_backoff_ms": 0,
            "tool_calls_ms": 254_200,
            "phases_ms": { "implement": 505_400 }
        });
        let line = format_timing_stdout_line_from_json(&json);
        assert!(line.contains("wall = 532.2s"));
        assert!(line.contains("llm_wait = 505.4s"));
        assert!(line.contains("tool_calls = 254.2s"));
        assert!(line.contains("implement = 505.4s"));
        assert!(!line.contains("agent_retry_backoff"));
    }

    #[test]
    fn phase_display_name_returns_alias_or_key() {
        let json: Value = serde_json::json!({
            "phase_display_names": { "implement": "raw" }
        });
        assert_eq!(phase_display_name(&json, "implement"), "raw");
        assert_eq!(phase_display_name(&json, "kpop"), "kpop");
    }
}
