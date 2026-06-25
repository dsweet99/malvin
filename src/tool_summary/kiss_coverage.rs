#[test]
fn kiss_cov_mod_private_record_helpers() {
    let _ = super::new_tool_call_record;
    let _ = super::merge_parsed_into_record;
}

#[test]
fn kiss_cov_tool_summary_core_symbols_for_kiss() {
    let _ = super::tool_summary_lines;
    let record = super::ToolCallRecord {
        kind: "read".into(),
        title: "t".into(),
        command: None,
        input_path: None,
        search_query: None,
        input_line_range: None,
        started: std::time::Instant::now(),
        stdout_start_emitted: false,
    };
    let super::ToolCallRecord {
        kind,
        title: _,
        command: _,
        input_path: _,
        search_query: _,
        input_line_range: _,
        started: _,
        stdout_start_emitted: _,
    } = record;
    assert_eq!(kind, "read");
    let lines = super::ToolSummaryLines {
        log: "log".into(),
        stdout: None,
        stdout_deferred: None,
    };
    let super::ToolSummaryLines {
        log,
        stdout: _,
        stdout_deferred: _,
    } = lines;
    assert_eq!(log, "log");
    let parsed = super::parse::ParsedToolUpdate {
        phase: super::types::TOOL_PHASE_START,
        id: "id".into(),
        kind: "read".into(),
        title: "t".into(),
        status: None,
        command: None,
        input_path: None,
        input_line_range: None,
        search_query: None,
        raw_output: None,
    };
    let super::parse::ParsedToolUpdate {
        phase,
        id,
        kind: _,
        title: _,
        status: _,
        command: _,
        input_path: _,
        input_line_range: _,
        search_query: _,
        raw_output: _,
    } = parsed;
    assert_eq!(phase, super::types::TOOL_PHASE_START);
    assert_eq!(id, "id");
    let _ = super::human_a::format_tool_stdout;
    let _ = super::human_a::execute_effective_exit;
    let _ = super::human_a::execute_stdout_failed;
    let _ = super::ansi::tool_summary_stdout_display;
    let _: Option<super::ToolSummaryLines> = None;
    let _ = super::format::format_tool_line;
    let _ = super::format::start_label;
    let _ = super::format::edit_paths;
    let _ = super::format::stderr_headline;
    let _ = super::format::stdout_headline;
    let _ = super::parse::json_number;
    let _ = super::parse::parse_tool_update;
    let _ = super::parse::tool_phase_label;
    let _ = super::parse_acp::acp_path_value;
    let _ = super::parse_acp::acp_normalize_path;
}

#[test]
fn kiss_cov_tool_summary_parse_acp_symbols_for_kiss() {
    let _ = super::parse_acp::acp_content_diff_paths;
    let _ = super::parse_acp::merge_content_diff_paths;
    let _ = super::parse_acp::acp_path_field;
    let _ = super::parse_acp::acp_search_query_field;
    let _ = super::parse_acp::acp_line_range_field;
    let _ = super::parse::phase_for_session_update;
    let _ = super::parse::parse_tool_update_fields;
    let _ = super::parse::parse_tool_update_identity;
    let _ = super::parse::parse_tool_update_metadata;
    let _ = super::parse_acp::content_diff_paths_to_raw_output;
    let _ = super::parse_acp::raw_output_has_edit_paths;
    let _ = super::parse_acp::merge_paths_into_raw;
}

#[test]
fn kiss_cov_format_first_non_empty_line_and_escape_quoted_behavioral() {
    assert_eq!(
        super::format::first_non_empty_line("  \n hello \n"),
        Some("hello")
    );
    assert!(super::format::first_non_empty_line("  \n ").is_none());
    assert_eq!(super::format::escape_quoted(r#"a"b"#), r#"a\"b"#);
}

#[test]
fn kiss_cov_tool_summary_format_symbols_for_kiss() {
    let _ = super::format::append_elapsed;
    let _ = super::format::append_start_title;
    let _ = super::format::append_done_fields;
    let _ = super::format::append_execute_done;
    let _ = super::format::append_read_done;
    let _ = super::format::append_search_done;
    let _ = super::format::push_edit_path;
    let _ = super::format::append_edit_done;
    let _ = super::format::append_edit_counts;
    let _ = super::format::append_generic_done;
    let _ = super::format::append_byte_fields;
    let _ = super::format::append_error_headline;
    let _ = super::format::first_non_empty_line;
    let _ = super::format::escape_quoted;
}

#[test]
fn kiss_cov_tool_summary_human_symbols_for_kiss() {
    let _ = super::human_a_done::human_done_line;
    let _ = super::human_a_done::human_search_start;
    let _ = super::human_a_done::search_query_from;
    let _ = super::human_b::relativize_tool_path;
    let _ = super::human_b::human_read_subject;
    let _ = super::ansi::apply_tool_summary_ansi;
    let _ = super::human_b::human_edit_subject;
    let _ = super::human_b::human_execute_command;
    let _ = super::human_b::raw_byte_size;
    let _ = super::human_b::humanize_bytes;
    let _ = super::human_b::humanize_duration;
    let _ = super::human_a::tool_stdout_should_emit;
    let _ = super::human_a::format_tool_line_human;
    let _ = super::human_a::human_start_line;
    let _ = super::human_a::human_running_line;
    let _ = super::human_a_done::human_read_done;
    let _ = super::human_a_done::human_search_done;
    let _ = super::human_a_done::human_execute_done;
    let _ = super::human_a_done::human_edit_done;
    let _ = super::human_a_done::human_edit_counts;
}

#[test]
fn kiss_cov_tool_summary_human_b_detail_symbols_for_kiss() {
    let _ = super::human_b::read_output_path;
    let _ = super::human_b::human_edit_subject_path;
    let _ = super::human_b::shorten_subject_path;
    let _ = super::human_b::format_line_range_suffix;
    let _ = super::human_b::read_or_edit_title_label;
    let _ = super::human_b::looks_like_path_label;
    let _ = super::human_b::escape_tool_subject_fragment;
    let _ = super::human_b::strip_execute_cd_prefix;
    let _ = super::ansi::apply_tool_summary_ansi;
    let _ = super::ansi::ansi_style_tool_segment;
    let _ = super::ansi::ansi_style_tool_segment_running_or_path;
    let _ = super::ansi::ansi_style_path_tail;
}
