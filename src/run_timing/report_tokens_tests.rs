use super::report_timing_line::format_timing_stdout_line_from_json;
use super::to_json_value;
use super::super::report_tokens_line::format_tokens_stdout_line_from_json;
use crate::openrouter_transport::ResponseUsage;
use crate::run_timing::RunTiming;

#[test]
fn run_timing_json_includes_tokens_block() {
    let mut r = RunTiming::default();
    r.record_mini_http_step(Some(&ResponseUsage {
        prompt_tokens: Some(100),
        completion_tokens: Some(20),
        total_tokens: Some(120),
        cost: Some(0.01),
    }));
    let json = to_json_value(&r);
    assert_eq!(json["tokens"]["steps"], 1);
    assert_eq!(json["tokens"]["tokens_in"], 100);
    assert_eq!(json["tokens"]["tokens_out"], 20);
}

#[test]
fn tokens_fields_on_separate_stdout_line_not_timing_line() {
    let mut r = RunTiming::default();
    r.record_mini_http_step(Some(&ResponseUsage {
        prompt_tokens: Some(50),
        completion_tokens: Some(10),
        total_tokens: Some(60),
        cost: None,
    }));
    let json = to_json_value(&r);
    let timing_line = format_timing_stdout_line_from_json(&json);
    assert!(!timing_line.contains("tokens_in"));
    assert!(!timing_line.contains("steps ="));
    let tokens_line = format_tokens_stdout_line_from_json(&json);
    assert!(tokens_line.starts_with("TOKENS:"));
    assert!(tokens_line.contains("steps = 1"));
    assert!(tokens_line.contains("tokens_in = 50"));
    assert!(tokens_line.contains("tokens_out = 10"));
}

#[test]
fn tokens_line_uses_na_when_never_observed() {
    let json = to_json_value(&RunTiming::default());
    let line = format_tokens_stdout_line_from_json(&json);
    assert_eq!(line, "TOKENS: steps = 0 tokens_in = n/a tokens_out = n/a");
}
