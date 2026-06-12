use std::time::Duration;

use malvin_mini::ResponseUsage;

use super::trace::format_mini_bash_tool_line;
use super::MiniTraceSink;
use crate::output::{is_log_timestamp_token, WHO_B, WHO_M};

fn stdout_log_tool_t_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            let Some((ts, rest)) = line.split_once(' ') else {
                return false;
            };
            is_log_timestamp_token(ts) && rest.starts_with("t|")
        })
        .collect()
}

fn test_io(no_tee: bool) -> crate::acp::AgentIoOptions {
    crate::acp::AgentIoOptions {
        force: false,
        no_tee,
        raw_output: !no_tee,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

#[test]
fn format_mini_bash_tool_line_matches_acp_execute_done_shape() {
    let line = format_mini_bash_tool_line("echo hi", 0, Duration::from_millis(12));
    assert_eq!(line, "Run echo hi · 12ms · ✓");
    let fail = format_mini_bash_tool_line("false", 1, Duration::from_millis(5));
    assert_eq!(fail, "Run false · 5ms · ✗ exit 1");
}

#[test]
fn format_mini_bash_tool_line_flattens_multiline_commands_to_single_line() {
    let command = "cat >> /path/file << 'EOF'\ncontent\nEOF";
    let line = format_mini_bash_tool_line(command, 0, Duration::from_millis(3));
    assert!(
        !line.contains('\n'),
        "tool summary must be one physical line; got {line:?}"
    );
    assert!(line.contains("Run "));
    assert!(line.contains("\\n"));
    assert!(line.ends_with("· 3ms · ✓"));
}

#[test]
fn mini_trace_writes_mini_llm_request_with_usage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(true),
    };
    sink.mini_llm_request(Some(&ResponseUsage {
        prompt_tokens: Some(1),
        completion_tokens: Some(2),
        total_tokens: Some(3),
        cost: Some(0.01),
    }));
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("mini_llm_request"));
    assert!(text.contains("cost"));
}

#[test]
fn mini_trace_writes_mini_bash_exec() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(true),
    };
    sink.mini_bash_exec("echo hi", 0, Duration::from_millis(1));
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("mini_bash_exec"));
}

#[test]
fn mini_stdout_emits_bash_tool_summary_with_t_tag() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(false),
    };
    sink.mini_bash_exec("echo hi", 0, Duration::from_millis(3));
    let text = std::fs::read_to_string(log_path).expect("stdout log");
    assert!(text.contains("t|"));
    assert!(text.contains("Run echo hi"));
    assert!(text.contains("✓"));
    crate::output::set_stdout_log_path(None);
}

#[test]
fn mini_stdout_multiline_bash_emits_single_t_tagged_line() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(false),
    };
    let command = "cat >> /path/file << 'EOF'\ncontent\nEOF";
    sink.mini_bash_exec(command, 0, Duration::from_millis(3));
    let text = std::fs::read_to_string(log_path).expect("stdout log");
    assert_eq!(
        text.lines().count(),
        1,
        "multiline command must log exactly one physical line; got {text:?}"
    );
    let t_lines = stdout_log_tool_t_lines(&text);
    assert_eq!(
        t_lines.len(),
        1,
        "multiline command must produce exactly one timestamped t| line; got {text:?}"
    );
    let payload = t_lines[0]
        .split_once(' ')
        .map_or(t_lines[0], |(_, rest)| rest);
    assert!(
        !payload.contains('\n'),
        "t| payload must not contain embedded newlines"
    );
    assert!(payload.contains("\\n"));
    assert!(payload.ends_with("· 3ms · ✓"));
    crate::output::set_stdout_log_path(None);
}

#[test]
fn mini_stdout_emits_assistant_with_m_tag_not_b_tag() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(false),
    };
    sink.mini_assistant("hello from mini");
    let text = std::fs::read_to_string(log_path).expect("stdout log");
    assert!(
        text.contains(&format!("{WHO_M}|")),
        "assistant text must use m| tag, got {text:?}"
    );
    assert!(
        !text.contains(&format!("{WHO_B}|")),
        "assistant text must not use b| tag, got {text:?}"
    );
    crate::output::set_stdout_log_path(None);
}

#[test]
fn mini_stdout_skips_assistant_when_no_tee() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log_path = tmp.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log_path.clone()));
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(true),
    };
    sink.mini_assistant("hidden");
    let text = std::fs::read_to_string(log_path).unwrap_or_default();
    assert!(text.is_empty(), "no_tee must suppress assistant stdout; got {text:?}");
    crate::output::set_stdout_log_path(None);
}

#[test]
fn mini_trace_writes_mini_assistant() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = MiniTraceSink {
        run_dir: Some(tmp.path().to_path_buf()),
        io: test_io(true),
    };
    sink.mini_assistant("hello assistant");
    let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(text.contains("mini_assistant"));
    assert!(text.contains("text_len"));
}
