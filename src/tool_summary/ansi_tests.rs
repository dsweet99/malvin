use super::{
    ansi_style_done_verb, ansi_style_running_verb, ansi_style_tool_name_verb,
    apply_tool_summary_ansi, is_byte_size_segment, split_outer_brackets, tool_line_colon_prefix,
};
use crate::terminal_palette::ANSI_DIM;
use crate::tool_summary::types::{ANSI_BOLD, ANSI_RESET, ansi_accent, ansi_tool_name};

fn covers_running_and_done_helpers() {
    assert!(ansi_style_running_verb("Reading path…").contains("Reading"));
    assert!(ansi_style_done_verb("Read path · 1ms").contains("Read"));
}

fn tool_line_colon_prefix_splits_leading_marker() {
    assert_eq!(tool_line_colon_prefix(":: Run x"), (":: ", "Run x"));
    assert_eq!(tool_line_colon_prefix("[Run x]"), ("[", "Run x"));
    assert_eq!(tool_line_colon_prefix("Run x"), ("", "Run x"));
}

fn ansi_style_tool_name_verb_wraps_verb_in_palette() {
    let styled = ansi_style_tool_name_verb("Edit");
    assert!(styled.contains("Edit"));
    assert!(styled.contains(ansi_tool_name()));
}

fn bracket_wrapped_running_line_bolds_run_verb() {
    let styled = apply_tool_summary_ansi("[Run echo hi…]");
    let run_verb = format!("{ANSI_BOLD}{}Run", ansi_tool_name());
    assert!(
        styled.contains(&run_verb),
        "expected dark bold on Run inside brackets; got {styled:?}"
    );
}

fn bracket_wrapped_done_line_bolds_run_verb() {
    let styled = apply_tool_summary_ansi("[Run echo hi · 1ms · ✓]");
    let run_verb = format!("{ANSI_BOLD}{}Run", ansi_tool_name());
    assert!(
        styled.contains(&run_verb),
        "expected dark bold on Run in done line; got {styled:?}"
    );
    assert!(styled.contains('['));
}

fn bracket_wrapped_reading_running_line_bolds_verb() {
    let styled = apply_tool_summary_ansi("[Reading ./src/foo.rs…]");
    let verb = format!("{ANSI_BOLD}{}Reading", ansi_tool_name());
    assert!(
        styled.contains(&verb),
        "expected dark bold on Reading; got {styled:?}"
    );
}

fn done_line_bolds_read_verb_without_colon_prefix() {
    let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
    let verb = format!("{ANSI_BOLD}{}Read", ansi_tool_name());
    assert!(
        styled.contains(&verb),
        "expected dark bold on Read; got {styled:?}"
    );
}

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

fn tool_time_segments_use_dim_grey() {
    let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 42ms");
    let dim = format!("{ANSI_DIM}42ms{ANSI_RESET}");
    assert!(styled.contains(&dim), "got {styled:?}");
}

fn tool_second_duration_segments_use_dim_grey() {
    let styled = apply_tool_summary_ansi("Run sleep 2 · 2.0s · ✓");
    let dim = format!("{ANSI_DIM}2.0s{ANSI_RESET}");
    assert!(styled.contains(&dim), "got {styled:?}");
}

fn comment_segment_with_s_uses_teal_not_dim() {
    let plain = "Run ls -ltr logs · List recent session logs befor · 8ms · ✓";
    let styled = apply_tool_summary_ansi(plain);
    let teal = format!(
        "{}List recent session logs befor{ANSI_RESET}",
        ansi_accent()
    );
    assert!(
        styled.contains(&teal),
        "comment must be teal; got {styled:?}"
    );
    let dim = format!("{ANSI_DIM}List recent session logs befor{ANSI_RESET}");
    assert!(
        !styled.contains(&dim),
        "comment must not be dim; got {styled:?}"
    );
}

fn tool_path_args_use_teal() {
    let styled = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
    let teal = format!("{}./src/foo.rs{ANSI_RESET}", ansi_accent());
    assert!(styled.contains(&teal), "got {styled:?}");
}

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
    assert!(dimmed.contains(&format!("{ANSI_DIM}{}[", ansi_tool_name())));
    assert!(dimmed.contains(&format!("{}]", ansi_tool_name())));
}

fn search_done_without_query_uses_dark_verb_not_teal() {
    let styled = apply_tool_summary_ansi("Search · matches");
    let verb = format!("{ANSI_BOLD}{}Search", ansi_tool_name());
    let teal = format!("{}Search", ansi_accent());
    assert!(
        styled.contains(&verb),
        "Search without query must use dark verb color; got {styled:?}"
    );
    assert!(
        !styled.contains(&teal),
        "Search verb must not be teal; got {styled:?}"
    );
}

fn glob_done_uses_dark_verb_not_teal() {
    let styled = apply_tool_summary_ansi("Glob **/* · 41ms");
    let verb = format!("{ANSI_BOLD}{}Glob", ansi_tool_name());
    let teal_verb = format!("{}Glob", ansi_accent());
    let teal_pat = format!("{}**/*{ANSI_RESET}", ansi_accent());
    assert!(
        styled.contains(&verb),
        "Glob must use tool-name color; got {styled:?}"
    );
    assert!(
        !styled.contains(&teal_verb),
        "Glob verb must not be teal; got {styled:?}"
    );
    assert!(
        styled.contains(&teal_pat),
        "glob pattern must stay teal; got {styled:?}"
    );
}

fn edit_search_and_editing_verbs_use_bold_dark() {
    let dark = format!("{ANSI_BOLD}{}", ansi_tool_name());
    for plain in [
        "Edit src/foo.rs · 1ms",
        "Editing src/foo.rs…",
        "Searching rg foo…",
        "Search rg needle · 1ms",
        "Search · matches",
        "Glob **/* · 41ms",
        "Glob",
    ] {
        assert!(
            apply_tool_summary_ansi(plain).contains(&dark),
            "got {plain:?}"
        );
    }
}

fn styled_running_and_done_lines_use_palette() {
    let running = apply_tool_summary_ansi("Reading ./src/foo.rs…");
    let done = apply_tool_summary_ansi("Read ./src/foo.rs · 1ms");
    assert!(running.contains("Reading"));
    assert!(done.contains("Read"));
}

#[test]
fn kiss_bundled_tool_summary_ansi_tests() {
    covers_running_and_done_helpers();
    tool_line_colon_prefix_splits_leading_marker();
    ansi_style_tool_name_verb_wraps_verb_in_palette();
    bracket_wrapped_running_line_bolds_run_verb();
    bracket_wrapped_done_line_bolds_run_verb();
    bracket_wrapped_reading_running_line_bolds_verb();
    done_line_bolds_read_verb_without_colon_prefix();
    byte_size_suffixes_use_dim_grey();
    tool_time_segments_use_dim_grey();
    tool_second_duration_segments_use_dim_grey();
    comment_segment_with_s_uses_teal_not_dim();
    tool_path_args_use_teal();
    split_outer_brackets_and_byte_size_segments();
    search_done_without_query_uses_dark_verb_not_teal();
    glob_done_uses_dark_verb_not_teal();
    edit_search_and_editing_verbs_use_bold_dark();
    styled_running_and_done_lines_use_palette();
}
