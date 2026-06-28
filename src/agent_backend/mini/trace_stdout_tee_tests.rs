use std::time::Duration;

use crate::output::{is_log_timestamp_token, WHO_B, WHO_M};

use super::trace_tests::{trace_sink, with_stdout_log_test_lock};

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

#[test]
fn mini_stdout_emits_bash_tool_summary_with_t_tag() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, false);
        sink.mini_bash_exec("echo hi", 0, Duration::from_millis(3), None);
        let text = std::fs::read_to_string(log_path).expect("stdout log");
        assert!(text.contains("t|"));
        assert!(text.contains("Run echo hi"));
        assert!(text.contains("✓"));
        crate::output::set_stdout_log_path(None);
    });
}

#[test]
fn mini_stdout_cat_emits_read_summary() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, false);
        sink.mini_bash_exec("cat README.md", 0, Duration::from_millis(3), None);
        let text = std::fs::read_to_string(log_path).expect("stdout log");
        assert!(text.contains("Read README.md"));
        crate::output::set_stdout_log_path(None);
    });
}

#[test]
fn mini_stdout_multiline_bash_emits_single_t_tagged_line() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, false);
        let command = "cat >> /path/file << 'EOF'\ncontent\nEOF";
        sink.mini_bash_exec(command, 0, Duration::from_millis(3), None);
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
        assert!(payload.starts_with("t|Edit "));
        assert!(payload.contains("/path/file"));
        assert!(payload.ends_with("· 3ms · ✓"));
        crate::output::set_stdout_log_path(None);
    });
}

#[test]
fn mini_stdout_emits_assistant_with_m_tag_not_b_tag() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, false);
        sink.stream_assistant_chunks("hello from mini");
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
    });
}

#[test]
fn mini_stdout_skips_assistant_when_no_tee() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, true);
        sink.stream_assistant_chunks("hidden");
        let text = std::fs::read_to_string(log_path).unwrap_or_default();
        assert!(text.is_empty(), "no_tee must suppress assistant stdout; got {text:?}");
        crate::output::set_stdout_log_path(None);
    });
}

#[test]
fn mini_reasoning_trace_without_stdout_when_thoughts_disabled() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, false);
        sink.mini_thought("audit-only thought");
        let stdout = std::fs::read_to_string(log_path).unwrap_or_default();
        assert!(
            stdout.is_empty(),
            "reasoning must not appear on stdout without --thoughts; got {stdout:?}"
        );
        let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(trace.contains("agent_thought_chunk"));
        assert!(trace.contains("audit-only thought"));
        crate::output::set_stdout_log_path(None);
    });
}
