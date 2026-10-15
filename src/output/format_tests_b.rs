use super::format_line_stdout_ansi;
use super::{
    WHO_B, WHO_P, init_stdout_style_for_test, print_outgoing_prompt_log, set_stdout_log_path,
};

#[test]
fn ansi_error_and_warning_color_entire_display_line() {
    use crate::terminal_palette::{ANSI_RESET, ansi_error, ansi_warning};

    let err = format_line_stdout_ansi(super::ERROR_WHO, "ACP failed");
    let coral_pos = err.find(ansi_error()).expect("coral");
    let reset_pos = err.rfind(ANSI_RESET).expect("reset");
    let payload_pos = err.find("ACP failed").expect("payload");
    assert_eq!(
        err.matches(ANSI_RESET).count(),
        1,
        "single trailing reset: {err:?}"
    );
    assert!(
        coral_pos < payload_pos && payload_pos < reset_pos,
        "error tag and payload must stay inside coral span: {err:?}"
    );
    let warn = format_line_stdout_ansi(super::WARNING_WHO, "disk low");
    let amber_pos = warn.find(ansi_warning()).expect("amber");
    let reset_pos = warn.rfind(ANSI_RESET).expect("reset");
    let payload_pos = warn.find("disk low").expect("payload");
    assert_eq!(
        warn.matches(ANSI_RESET).count(),
        1,
        "single trailing reset: {warn:?}"
    );
    assert!(
        amber_pos < payload_pos && payload_pos < reset_pos,
        "warning tag and payload must stay inside amber span: {warn:?}"
    );
}

#[test]
fn ansi_thought_tag_uses_uniform_dim_grey() {
    use crate::terminal_palette::{ANSI_DIM, ansi_tool_name, ansi_who_tag};

    let line = format_line_stdout_ansi(WHO_B, "fail with max_abs=1.0.");
    assert!(
        line.contains("b| "),
        "thought display must space after pipe; got {line:?}"
    );
    assert!(line.contains(ANSI_DIM));
    assert!(!line.contains(ansi_who_tag()));
    assert!(!line.contains(ansi_tool_name()));
    let dim_start = line.find(ANSI_DIM).expect("dim");
    assert!(line[dim_start..].contains("fail with max_abs=1.0."));
}

#[test]
fn outgoing_prompt_log_uses_p_tag_and_drops_md_extension() {
    let _guard = super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    init_stdout_style_for_test(false);
    crate::output::enable_stdout_capture();
    print_outgoing_prompt_log("bug_fix", "bug_fix.md");
    print_outgoing_prompt_log("router_initial", "header.md+router_a.md");
    let live = crate::output::take_captured_stdout();
    set_stdout_log_path(None);
    let text = std::fs::read_to_string(path).expect("read stdout log");
    let delim = super::format_who_tag_delim(WHO_P);
    assert!(
        text.contains(&format!("{delim}bug_fix")),
        "expected p| and stem without .md: {text:?}"
    );
    assert!(
        text.contains(&format!("{delim}header")) && text.contains(&format!("{delim}router_a")),
        "expected each internal prompt on its own line: {text:?}"
    );
    assert!(
        !text.contains('+') && !text.contains(".md"),
        "prompt file log must split stems and drop .md: {text:?}"
    );
    let header_at = live.find("p|header").expect("live header");
    let router_at = live.find("p|router_a").expect("live router_a");
    assert!(
        live.contains("p|bug_fix") && header_at < router_at && !live.contains('+'),
        "each internal prompt must show on the live terminal: {live:?}"
    );
}
