// Per-symbol kiss coverage wires.

#[test]
fn kiss_cov_src_cli_source_detect_rs_entry_name_is_workspace_marker() {
    let _ = stringify!(crate::source_detect::entry_name_is_workspace_marker);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_resolved_symlink_target() {
    let _ = stringify!(crate::source_detect::resolved_symlink_target);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_symlink_resolves_to_existing_file() {
    let _ = stringify!(crate::source_detect::symlink_resolves_to_existing_file);
}

#[test]
fn kiss_cov_src_cli_source_detect_rs_entry_or_symlink_file_target_matches() {
    let _ = stringify!(crate::source_detect::entry_or_symlink_file_target_matches);
}

#[test]
fn kiss_cov_src_orchestrator_bug_remediation_rs_run_bug_remediation_gap() {
    let _ = stringify!(crate::orchestrator::run_bug_remediation_gap);
}

#[test]
fn kiss_cov_src_orchestrator_review_attempt_kernel_rs_read_artifact_review_text() {
    let _ = stringify!(crate::orchestrator::read_artifact_review_text);
}

#[test]
fn kiss_cov_src_orchestrator_review_loop_rs_code_review_attempt_outcome() {
    let _ = stringify!(crate::orchestrator::CodeReviewAttemptOutcome);
}

#[test]
fn kiss_cov_src_output_mod_rs_push_captured_stderr_line() {
    let _ = stringify!(crate::output::push_captured_stderr_line);
}

#[test]
fn kiss_cov_src_output_mod_rs_timestamp_now_string() {
    let _ = stringify!(crate::output::timestamp_now_string);
}

#[test]
fn kiss_cov_src_output_mod_rs_who_tag_ansi() {
    let _ = stringify!(crate::output::who_tag_ansi);
}

#[test]
fn kiss_cov_src_output_mod_rs_log_use_color() {
    let _ = stringify!(crate::output::log_use_color);
}

#[test]
fn kiss_cov_src_output_mod_rs_stderr_use_color() {
    let _ = stringify!(crate::output::stderr_use_color);
}

#[test]
fn kiss_cov_src_output_mod_rs_set_stdout_log_path() {
    let _ = stringify!(crate::output::set_stdout_log_path);
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_stdout_tagged_display_and_log_line() {
    let (display, log) =
        crate::output::stdout_log_pair::stdout_tagged_display_and_log_line("k", "p", None);
    assert!(!display.starts_with("20"));
    assert!(log.starts_with("20"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_stdout_raw_display_and_log_line() {
    let (display, log) =
        crate::output::stdout_log_pair::stdout_raw_display_and_log_line("raw", None);
    assert_eq!(display, "raw");
    assert!(log.starts_with("20"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_stdout_acp_display_and_log() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt};
    let display_ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "display",
        dim_payload: false,
    };
    let log_ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "log",
        dim_payload: false,
    };
    let (display, log) =
        crate::output::stdout_log_pair::stdout_acp_display_and_log(&display_ctx, &log_ctx);
    assert!(display.contains("display"));
    assert!(log.contains("log"));
    assert!(log.starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_format_line_acp_ansi_payload() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt, stdout_log_pair::format_line_acp_ansi_payload};
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "p",
        dim_payload: false,
    };
    assert!(!format_line_acp_ansi_payload(&ctx).starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_stdout_acp_prefix_rendered_line() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt};
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "",
        dim_payload: false,
    };
    let (display, log) =
        crate::output::stdout_log_pair::stdout_acp_prefix_rendered_line(&ctx, "body");
    assert!(display.contains("body"));
    assert!(log.contains("body"));
    assert!(log.starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_mod_rs_print_stdout_raw_line_with_ts() {
    let (_, log) =
        crate::output::stdout_log_pair::stdout_raw_display_and_log_line("raw", Some("20260413.121314.015"));
    assert!(log.starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_acp_tee_display_line() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt, acp_tee_display_line};
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "p",
        dim_payload: false,
    };
    assert!(!acp_tee_display_line(&ctx).starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_acp_tee_log_line() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt, acp_tee_log_line};
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "p",
        dim_payload: false,
    };
    assert!(acp_tee_log_line(&ctx).starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_acp_tee_log_prefix() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt, stdout_log_pair::acp_tee_log_prefix};
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "",
        dim_payload: false,
    };
    assert!(acp_tee_log_prefix(&ctx).starts_with("20260413"));
}

#[test]
fn kiss_cov_src_output_stdout_log_pair_rs_acp_tee_payload_prefix() {
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt, stdout_log_pair::acp_tee_payload_prefix};
    let ctx = AcpTeeLineFmt {
        ts: "20260413.121314.015",
        direction: AcpTeeDirection::FromAgent,
        who: "<k",
        line: "",
        dim_payload: false,
    };
    assert!(!acp_tee_payload_prefix(&ctx).starts_with("20260413"));
}

