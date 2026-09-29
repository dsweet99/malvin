use super::{
    REQUEST_INLINE_MAX_BYTES, RPI_EXP_LOG_NOTE, format_exp_log_note, format_request_inline,
    format_request_inline_text,
};

#[test]
fn exp_log_note_only_for_rpi_models() {
    assert_eq!(format_exp_log_note("rpi:openai/gpt-4o"), RPI_EXP_LOG_NOTE);
    assert_eq!(format_exp_log_note("pi:openai/gpt-4o"), "");
    assert_eq!(format_exp_log_note("cursor:auto"), "");
}
use std::path::Path;

#[test]
fn short_request_is_inlined_whole_in_a_fence() {
    let out = format_request_inline_text("Fix A2.\n", Path::new("/logs/plan_x.md"));
    assert!(out.contains("```text\nFix A2.\n```"));
    assert!(!out.contains("cat /logs/plan_x.md"));
}

#[test]
fn fence_is_longer_than_backtick_runs_in_the_request() {
    let out = format_request_inline_text("see ```rust\nx\n```", Path::new("/p.md"));
    assert!(out.contains("````text\n"));
}

#[test]
fn empty_request_gives_nothing() {
    assert_eq!(format_request_inline_text(" \n", Path::new("/p.md")), "");
}

#[test]
fn long_request_is_cut_on_a_char_boundary_with_a_cat_pointer() {
    let text = "é".repeat(REQUEST_INLINE_MAX_BYTES);
    let out = format_request_inline_text(&text, Path::new("/logs/plan_x.md"));
    assert!(out.contains("Read the rest with `cat /logs/plan_x.md` via the `bash` tool."));
    assert!(out.len() < REQUEST_INLINE_MAX_BYTES + 400);
}

#[test]
fn only_rpi_models_get_inline_text() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "inline-marker").expect("write");
    assert!(format_request_inline("rpi:local/ollama/llama3.2:3b", &plan).contains("inline-marker"));
    assert_eq!(format_request_inline("cursor:auto", &plan), "");
    assert_eq!(format_request_inline("pi:openai/gpt-4o", &plan), "");
    assert_eq!(
        format_request_inline("rpi:openai/gpt-4o", &tmp.path().join("missing.md")),
        ""
    );
}
