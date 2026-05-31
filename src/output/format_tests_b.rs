use super::format_line_stdout_ansi;
use super::{WHO_B, WHO_U, init_stdout_style, print_outgoing_prompt_log, set_stdout_log_path};

#[test]
fn ansi_error_and_warning_color_entire_display_line() {
    use crate::terminal_palette::{ansi_tool_amber, ansi_tool_coral, ANSI_RESET};

    let err = format_line_stdout_ansi(super::ERROR_WHO, "ACP failed");
    let coral_pos = err.find(ansi_tool_coral()).expect("coral");
    let reset_pos = err.rfind(ANSI_RESET).expect("reset");
    let payload_pos = err.find("ACP failed").expect("payload");
    assert_eq!(err.matches(ANSI_RESET).count(), 1, "single trailing reset: {err:?}");
    assert!(
        coral_pos < payload_pos && payload_pos < reset_pos,
        "error tag and payload must stay inside coral span: {err:?}"
    );
    let warn = format_line_stdout_ansi(super::WARNING_WHO, "disk low");
    let amber_pos = warn.find(ansi_tool_amber()).expect("amber");
    let reset_pos = warn.rfind(ANSI_RESET).expect("reset");
    let payload_pos = warn.find("disk low").expect("payload");
    assert_eq!(warn.matches(ANSI_RESET).count(), 1, "single trailing reset: {warn:?}");
    assert!(
        amber_pos < payload_pos && payload_pos < reset_pos,
        "warning tag and payload must stay inside amber span: {warn:?}"
    );
}

#[test]
fn ansi_thought_tag_uses_uniform_dim_grey() {
    use crate::terminal_palette::{ansi_tool_dark, ansi_tool_navy, ANSI_DIM};

    let line = format_line_stdout_ansi(WHO_B, "fail with max_abs=1.0.");
    assert!(line.contains(ANSI_DIM));
    assert!(!line.contains(ansi_tool_navy()));
    assert!(!line.contains(ansi_tool_dark()));
    let dim_start = line.find(ANSI_DIM).expect("dim");
    assert!(line[dim_start..].contains("fail with max_abs=1.0."));
}

#[test]
fn outgoing_prompt_log_who_tag_uses_stem_bracket_keeps_md() {
    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    init_stdout_style(true);
    print_outgoing_prompt_log("bug_fix", "bug_fix.md");
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read stdout log");
    let prefix = super::format_who_tag_prefix(WHO_U);
    assert!(
        text.contains(&format!("{prefix}[bug_fix.md...]")),
        "expected user who tag and .md bracket payload: {text:?}"
    );
    assert!(
        !text.contains(">bug_fix"),
        "who tag must not include legacy directional stem: {text:?}"
    );
}

#[test]
fn defer_stdout_hooks_route_through_active_sink() {
    use std::path::PathBuf;
    use std::sync::Arc;

    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let shared = Arc::new(std::sync::Mutex::new(
        crate::deferred_log::DeferredLogSink::for_prompt("fmt_hook".to_string(), PathBuf::new())
            .expect("defer sink"),
    ));
    crate::deferred_log::register_active_sink(Arc::clone(&shared));
    crate::deferred_log::install_stdout_hooks();
    assert!(super::try_defer_tagged_stdout("d", "l"));
    assert!(crate::output::stdout_defer::try_defer_heartbeat("hb-d", "hb-l"));
    crate::deferred_log::unregister_active_sink();
    assert!(!super::try_defer_tagged_stdout("d", "l"));
    assert!(!crate::output::stdout_defer::try_defer_heartbeat("hb-d", "hb-l"));
}
