use std::time::Duration;

use malvin_mini::ResponseUsage;

use super::trace::format_mini_bash_tool_line;
use super::MiniTraceSink;

fn test_io(no_tee: bool) -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        force: false,
        no_tee,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

pub(crate) fn trace_sink(tmp: &tempfile::TempDir, no_tee: bool) -> MiniTraceSink {
    MiniTraceSink::new(Some(tmp.path().to_path_buf()), test_io(no_tee))
}

fn parse_trace_lines(path: &std::path::Path) -> Vec<serde_json::Value> {
    let text = std::fs::read_to_string(path).expect("trace");
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|line| serde_json::from_str(line).expect("valid jsonl"))
        .collect()
}

#[test]
fn format_mini_bash_tool_line_run_fallback_matches_acp_execute_done_shape() {
    let line = format_mini_bash_tool_line("echo hi", 0, Duration::from_millis(12), None);
    assert_eq!(line, "Run echo hi · 12ms · ✓");
    let fail = format_mini_bash_tool_line("false", 1, Duration::from_millis(5), None);
    assert_eq!(fail, "Run false · 5ms · ✗ exit 1");
}

#[test]
fn format_mini_bash_tool_line_classifies_cat_as_read() {
    let line = format_mini_bash_tool_line("cat file.txt", 0, Duration::from_millis(3), None);
    assert!(line.starts_with("Read file.txt"));
}

#[test]
fn format_mini_bash_tool_line_flattens_multiline_commands_to_single_line() {
    let command = "cat >> /path/file << 'EOF'\ncontent\nEOF";
    let line = format_mini_bash_tool_line(command, 0, Duration::from_millis(3), None);
    assert!(
        !line.contains('\n'),
        "tool summary must be one physical line; got {line:?}"
    );
    assert!(line.starts_with("Edit /path/file"));
    assert!(line.contains("\\n") || line.contains("/path/file"));
    assert!(line.ends_with("· 3ms · ✓"));
}

#[test]
fn mini_trace_acp_schema_llm_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.mini_llm_request(Some(&ResponseUsage {
        prompt_tokens: Some(1),
        completion_tokens: Some(2),
        total_tokens: Some(3),
        cost: Some(0.01),
    }));
    let records = parse_trace_lines(&tmp.path().join("trace.jsonl"));
    assert!(!records.is_empty());
    assert!(records.iter().all(|r| r.get("direction").is_some()));
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("cost"));
    assert!(!text.contains("mini_llm_request"));
}

#[test]
fn mini_trace_acp_schema_bash_exec() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.mini_bash_exec("echo hi", 0, Duration::from_millis(1), None);
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("direction"));
    assert!(text.contains("tool_call"));
    assert!(!text.contains("mini_bash_exec"));
}

pub(crate) fn with_stdout_log_test_lock<F: FnOnce()>(f: F) {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    f();
}

#[test]
fn mini_no_tee_still_writes_acp_trace_assistant() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.stream_assistant_chunks("trace only");
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("agent_message_chunk"));
    assert!(text.contains("trace only"));
}

#[test]
fn mini_no_tee_still_writes_acp_trace_bash() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.mini_bash_exec("echo hi", 0, Duration::from_millis(1), None);
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("tool_call"));
}

#[test]
fn mini_trace_acp_schema_assistant_full_text() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.stream_assistant_chunks("hello assistant");
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("agent_message_chunk"));
    assert!(text.contains("hello assistant"));
    assert!(!text.contains("text_len"));
    assert!(!text.contains("mini_assistant"));
}

#[test]
fn mini_trace_outgoing_prompt_has_direction_out() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.log_outgoing_prompt("constraints\n\nuser prompt");
    let records = parse_trace_lines(&tmp.path().join("trace.jsonl"));
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["direction"], "out");
}

#[test]
fn mini_no_tee_still_writes_acp_trace_reasoning() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = trace_sink(&tmp, true);
    sink.mini_thought("hidden reasoning blob");
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("agent_thought_chunk"));
    assert!(text.contains("hidden reasoning blob"));
}
