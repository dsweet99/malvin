use std::time::Duration;

use crate::tool_summary::{
    classify_bash_command, format_classified_tool_line, ClassifiedToolLineInput,
};

use super::trace_tests::{trace_sink, with_stdout_log_test_lock};

#[test]
fn format_bash_tool_line_appends_first_30_chars_of_comment() {
    let comment = "List recent session logs before editing the parser module";
    let line = format_classified_tool_line(ClassifiedToolLineInput {
        kind: classify_bash_command("ls -ltr logs"),
        command: "ls -ltr logs",
        exit_code: 0,
        elapsed: Duration::from_millis(8),
        comment: Some(comment),
    });
    assert_eq!(line, "Run ls -ltr logs · List recent session logs befor · 8ms · ✓");
}

#[test]
fn mini_stdout_emits_bash_tool_comment_on_t_line() {
    with_stdout_log_test_lock(|| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log_path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(log_path.clone()));
        let sink = trace_sink(&tmp, false);
        sink.mini_bash_exec(
            "ls -ltr logs",
            0,
            Duration::from_millis(5),
            Some("List recent session logs"),
        );
        let text = std::fs::read_to_string(log_path).expect("stdout log");
        assert!(
            text.contains("List recent session logs"),
            "stdout must include comment prefix; got {text:?}"
        );
        crate::output::set_stdout_log_path(None);
    });
}
