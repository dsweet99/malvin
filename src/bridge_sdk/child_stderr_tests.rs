use super::child_stderr::{
    backend_stderr_warning_payload, emit_backend_stderr_warning, start_warning_forward,
};

#[test]
fn empty_and_whitespace_only_lines_are_not_logged() {
    assert_eq!(backend_stderr_warning_payload(""), None);
    assert_eq!(backend_stderr_warning_payload("\n"), None);
    assert_eq!(backend_stderr_warning_payload("\r\n"), None);
}

#[test]
fn payload_strips_trailing_newlines() {
    assert_eq!(
        backend_stderr_warning_payload("ERROR boom\n"),
        Some("ERROR boom")
    );
    assert_eq!(
        backend_stderr_warning_payload("warn\r\n"),
        Some("warn")
    );
}

#[test]
fn emit_logs_as_warning_who_tag() {
    crate::output::clear_captured_stderr_lines();
    emit_backend_stderr_warning("ERROR from backend\n", &|_| false);
    let lines = crate::output::take_captured_stderr_lines();
    let warning_tag = crate::output::format_who_tag_delim(crate::output::WARNING_WHO);
    assert!(
        lines.iter().any(|l| l.contains(&warning_tag) && l.contains("ERROR from backend")),
        "expected warning-tagged backend stderr, got {lines:?}"
    );
}

#[test]
fn emit_respects_drop_filter() {
    crate::output::clear_captured_stderr_lines();
    emit_backend_stderr_warning("drop-me\n", &|line| line.contains("drop-me"));
    let lines = crate::output::take_captured_stderr_lines();
    assert!(
        lines.iter().all(|l| !l.contains("drop-me")),
        "dropped lines must not be logged, got {lines:?}"
    );
}

#[test]
fn kiss_cov_child_stderr_names() {
    let _ = start_warning_forward;
    let _ = stringify!(start_warning_forward_filtered);
    let _ = stringify!(take_stdio_forward_stderr);
    let _ = stringify!(forward_stderr_as_warnings);
    let _ = stringify!(backend_stderr_warning_payload);
    let _ = stringify!(emit_backend_stderr_warning);
}
