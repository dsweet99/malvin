use super::{
    ansi_style_done_verb, ansi_style_running_verb, ansi_style_sand_verb, apply_tool_summary_ansi,
    is_byte_size_segment, split_outer_brackets, tool_line_colon_prefix,
};
use crate::terminal_palette::ANSI_DIM;
use crate::tool_summary::types::{ANSI_BOLD, ANSI_RESET, ANSI_TOOL_CREAM, ANSI_TOOL_SAND};

#[test]
fn covers_running_and_done_helpers() {
    assert!(ansi_style_running_verb("Reading path…").contains("Reading"));
    assert!(ansi_style_done_verb("Read path · 1ms").contains("Read"));
}

#[test]
fn tool_line_colon_prefix_splits_leading_marker() {
    assert_eq!(tool_line_colon_prefix(":: Run x"), (":: ", "Run x"));
    assert_eq!(tool_line_colon_prefix("[Run x]"), ("[", "Run x"));
    assert_eq!(tool_line_colon_prefix("Run x"), ("", "Run x"));
}

#[test]
fn ansi_style_sand_verb_wraps_verb_in_palette() {
    let styled = ansi_style_sand_verb("Edit");
    assert!(styled.contains("Edit"));
    assert!(styled.contains(ANSI_TOOL_SAND));
}

#[test]
fn bracket_wrapped_running_line_bolds_run_verb() {
    let styled = apply_tool_summary_ansi("[Run echo hi…]");
    let run_verb = format!("{ANSI_BOLD}{ANSI_TOOL_SAND}Run");
    assert!(
        styled.contains(&run_verb),
        "expected sand bold on Run inside brackets; got {styled:?}"
    );
}

#[test]
fn bracket_wrapped_done_line_bolds_run_verb() {
    let styled = apply_tool_summary_ansi("[Run echo hi · 1ms · ✓]");
    let run_verb = format!("{ANSI_BOLD}{ANSI_TOOL_SAND}Run");
    assert!(
        styled.contains(&run_verb),
        "expected sand bold on Run in done line; got {styled:?}"
    );
    assert!(styled.contains('['));
}

#[test]
fn bracket_wrapped_reading_running_line_bolds_verb() {
    let styled = apply_tool_summary_ansi("[Reading ./src/foo.rs…]");
    let verb = format!("{ANSI_BOLD}{ANSI_TOOL_SAND}Reading");
    assert!(
        styled.contains(&verb),
        "expected sand bold on Reading; got {styled:?}"
    );
}

#[test]
fn done_line_bolds_read_verb_without_colon_prefix() {
    let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
    let verb = format!("{ANSI_BOLD}{ANSI_TOOL_SAND}Read");
    assert!(
        styled.contains(&verb),
        "expected sand bold on Read; got {styled:?}"
    );
}

#[test]
fn byte_size_suffixes_use_dim_grey() {
    for (plain, segment) in [
        ("Read file.bbb · 123 B · 1ms", "123 B"),
        ("Read x · 4 KB · 1ms", "4 KB"),
        ("Read x · 2 MB · 1ms", "2 MB"),
    ] {
        let styled = apply_tool_summary_ansi(plain);
        let dim = format!("{ANSI_DIM}{segment}{ANSI_RESET}");
        assert!(styled.contains(&dim), "got {styled:?}");
    }
}

#[test]
fn tool_time_segments_use_dim_grey() {
    let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 42ms");
    let dim = format!("{ANSI_DIM}42ms{ANSI_RESET}");
    assert!(styled.contains(&dim), "got {styled:?}");
}

#[test]
fn tool_path_args_use_cream() {
    let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
    let cream = format!("{ANSI_TOOL_CREAM}./src/foo.rs{ANSI_RESET}");
    assert!(styled.contains(&cream), "got {styled:?}");
}

#[test]
fn split_outer_brackets_and_byte_size_segments() {
    assert_eq!(split_outer_brackets("[Read x]"), ("[", "Read x", "]"));
    assert_eq!(split_outer_brackets("plain"), ("", "plain", ""));
    assert!(is_byte_size_segment("123 B"));
    assert!(!is_byte_size_segment("foo"));
    let styled = apply_tool_summary_ansi("[Read x · 4 KB]");
    assert!(styled.contains('[') && styled.contains(']'));
    let dimmed = format!(
        "{ANSI_DIM}{}{ANSI_RESET}",
        apply_tool_summary_ansi("[Run echo hi · 1ms · ✓]")
    );
    assert!(dimmed.contains(&format!("{ANSI_DIM}{ANSI_TOOL_SAND}[")));
    assert!(dimmed.contains(&format!("{ANSI_TOOL_SAND}]")));
}

#[test]
fn edit_search_and_editing_verbs_use_bold_sand() {
    let sand = format!("{ANSI_BOLD}{ANSI_TOOL_SAND}");
    for plain in [
        "Edit src/foo.rs · 1ms",
        "Editing src/foo.rs…",
        "Searching rg foo…",
        "Search rg needle · 1ms",
    ] {
        assert!(
            apply_tool_summary_ansi(plain).contains(&sand),
            "got {plain:?}"
        );
    }
}

#[test]
fn styled_running_and_done_lines_use_palette() {
    let running = apply_tool_summary_ansi("Reading ./src/foo.rs…");
    let done = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
    assert!(running.contains("Reading"));
    assert!(done.contains("Read"));
}
