use std::time::Duration;

use crate::output::{
    enable_stdout_capture, set_do_dm_stdout_mode, take_captured_stdout, DM_END, DM_START, WHO_M,
};

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
fn mini_do_plain_stdout_suppresses_bash_fence_assistant_text() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let mut sink = trace_sink(&tmp, false);
        sink.plain_lines = true;
        sink.record_assistant_audit("```bash\ncat plan_dco.md\n```");
        let text = std::fs::read_to_string(log_path).unwrap_or_default();
        assert!(
            text.is_empty(),
            "plain do must suppress bash fence assistant text; got {text:?}"
        );
        let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(
            trace.contains("agent_message_chunk"),
            "trace must still record assistant chunks; got {trace:?}"
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

#[test]
fn mini_do_dm_stream_emits_only_fence_body_on_process_stdout() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        let mut sink = trace_sink(&tmp, false);
        sink.plain_lines = true;
        let reply = format!("noise outside\n{DM_START}\nIt is noon EDT.\n{DM_END}\nmore noise");
        sink.stream_assistant_chunks(&reply);
        let captured = take_captured_stdout();
        set_do_dm_stdout_mode(false);
        assert_eq!(
            captured.trim(),
            "It is noon EDT.",
            "process stdout must be DM body only; got {captured:?}"
        );
        assert!(
            !captured.contains("MALVIN_DM_"),
            "fence markers must not appear on process stdout; got {captured:?}"
        );
        assert!(
            !captured.contains("noise"),
            "non-DM chatter must not appear on process stdout; got {captured:?}"
        );
        crate::output::set_stdout_log_path(None);
    });
}

#[test]
fn mini_do_dm_feed_from_bash_audit_emits_body_without_bash_tee() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        let mut sink = trace_sink(&tmp, false);
        sink.plain_lines = true;
        let reply = format!(
            "```bash\ndate\n```\n{DM_START}\nWed 29 Jul 2026 11:41 AM EDT\n{DM_END}\n"
        );
        sink.record_assistant_audit(&reply);
        sink.feed_do_dm_assistant_text(&reply);
        let captured = take_captured_stdout();
        set_do_dm_stdout_mode(false);
        let log_text = std::fs::read_to_string(&log_path).unwrap_or_default();
        assert_eq!(
            captured.trim(),
            "Wed 29 Jul 2026 11:41 AM EDT",
            "DM body must reach process stdout; got {captured:?}"
        );
        assert!(
            !captured.contains("```") && !captured.contains("date"),
            "bash fence must not reach process stdout; got {captured:?}"
        );
        assert!(
            log_text.is_empty(),
            "plain bash+DM audit path must not tee to stdout.log; got {log_text:?}"
        );
        crate::output::set_stdout_log_path(None);
    });
}
