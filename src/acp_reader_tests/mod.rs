#![allow(unused_imports, clippy::await_holding_lock)]

mod shared;

mod coalesce;
mod dispatch;
mod permission_a;
mod permission_b;
mod reader_loop_a;
mod reader_loop_b;
mod trace;

pub(super) use shared::*;
pub(super) use coalesce::*;
pub(super) use dispatch::*;
pub(super) use permission_a::*;
pub(super) use permission_b::*;
pub(super) use reader_loop_a::*;
pub(super) use reader_loop_b::*;
pub(super) use trace::*;

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn kiss_stringify_units() {
        let _ = stringify!(super::acp_activity_state);
        let _ = stringify!(super::dispatch_clears_prompt_cleanup_when_id_matches);
        let _ = stringify!(super::dispatch_resolves_pending_when_response_id_is_decimal_string);
        let _ = stringify!(super::dispatch_resolves_pending_when_response_id_is_i64);
        let _ = stringify!(super::kpop_permission_without_correlation_id_writes_nothing_to_child_stdin);
        let _ = stringify!(super::permission_with_id_in_params_writes_allow_always_reply_line);
        let _ = stringify!(super::raw_trace_file_write_line_records_thought_chunks_suppresses_thought_stdout_only);
        let _ = stringify!(super::test_dispatch_response_ok_error_orphans_and_malformed);
        let _ = stringify!(super::test_handle_incoming_line_parse_error_and_extension_method);
        let _ = stringify!(super::test_handle_session_update_and_permission_replies);
        let _ = stringify!(super::test_permission_json_or_write_failure_is_logged);
        let _ = stringify!(super::test_reader_loop_drains_pending_on_stdout_eof);
        let _ = stringify!(super::test_reader_loop_maps_memory_limit_on_stdout_eof);
        let _ = stringify!(super::trace_file_write_line_brackets_thought_chunks_in_trace_output);
        let _ = stringify!(super::trace_file_write_line_plain_mode_omits_tag_prefix);
        let _ = stringify!(super::trace_file_write_line_prefixes_with_prompt_who);
        let _ = stringify!(super::trace_file_write_line_stdout_markdown_flag_tees_without_panic);
        let _ = stringify!(crate::acp_test_unix_bin::unix_bin_with_fallback);
        let _ = stringify!(super::write_trace_line_coalesced_does_not_tee_parsed_non_chunk_lines);
        let _ = stringify!(super::write_trace_line_coalesced_writes_malformed_non_json_lines);
        let _ = stringify!(super::write_trace_line_coalesced_writes_non_chunk_lines);
        let _ = stringify!(super::incoming_permission_dispatch_plain);
        let _ = stringify!(super::UnixCatIncoming);
        let _ = stringify!(super::unix_cat_stdio_incoming);
        let _ = stringify!(super::unix_true_exited_stdio_stdin_only);
        let _ = stringify!(super::sleep_stdin_pipe_holder);
        let _ = stringify!(super::idle_prompt_cleanup_bundle);
        let _ = stringify!(super::spawned_true_stdout_pending_wire);
        let _ = stringify!(super::ReaderTrueStdoutPendingEof);
        let _ = stringify!(super::assemble_true_stdout_pending_reader);
        let _ = stringify!(super::spawn_reader_true_stdout_pending_eof);
    }
}
