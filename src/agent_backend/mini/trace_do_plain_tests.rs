use std::time::Duration;

use crate::output::WHO_M;

use super::trace_tests::{trace_sink, with_stdout_log_test_lock};

#[test]
fn mini_do_plain_stdout_emits_untagged_assistant() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let mut sink = trace_sink(&tmp, false);
        sink.plain_lines = true;
        sink.stream_assistant_chunks("Hello. What would you like to work on?");
        let text = std::fs::read_to_string(log_path).expect("stdout log");
        assert!(
            text.contains("Hello. What would you like to work on?"),
            "plain do must emit assistant text; got {text:?}"
        );
        assert!(
            !text.contains(&format!("{WHO_M}|")),
            "plain do must not use m| tag; got {text:?}"
        );
        crate::output::set_stdout_log_path(None);
    });
}

#[test]
fn mini_do_plain_stdout_suppresses_bash_tool_tee() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let mut sink = trace_sink(&tmp, false);
        sink.plain_lines = true;
        sink.mini_bash_exec("echo hi", 0, Duration::from_millis(3), None);
        let text = std::fs::read_to_string(log_path).unwrap_or_default();
        assert!(
            text.is_empty(),
            "plain do must suppress tool summary on stdout; got {text:?}"
        );
        let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(trace.contains("tool_call"), "trace must still record bash");
        crate::output::set_stdout_log_path(None);
    });
}
