pub(crate) mod acp_tee;
mod acp_tee_markdown;
mod code_fence;
pub use code_fence::CodeFence;
mod code_hl;
mod code_lex;
mod code_paint;
mod dm_attachment;
mod dm_on_emit_render;
mod dm_on_emit_run;
mod dm_relay;
pub use dm_relay::set_dm_emit_enabled;
mod dm_stderr;
mod do_dm_emit;
mod do_dm_filter;
mod do_dm_on_emit;
pub mod heartbeat_hooks;
mod line_format;
mod md_table;
mod prompt_log;
mod sentinel;
pub(crate) mod stderr_log;
mod stdout_display;
mod stdout_display_md;
mod stdout_emit;
mod stdout_format;
pub(crate) mod stdout_heartbeat;
pub(crate) mod stdout_log_pair;
mod stdout_open_line;
pub(crate) use stdout_open_line::set_agent_text_line_open;
mod stdout_render;
pub(crate) mod stdout_tee_env;
mod stdout_terminal;
mod style;
pub(crate) mod terminal_wrap;
mod test_modules;
mod who_tag;

#[cfg(test)]
pub(crate) use stdout_render::emit_stdout_rendered_immediate;

pub(crate) use stdout_format::{
    format_heartbeat_stdout_ansi, format_line_stdout, format_line_stdout_ansi, logical_lines,
};

#[cfg(test)]
pub(crate) use dm_relay::mark_line;
pub(crate) use dm_relay::parse_dm_emit_mark;
#[cfg(test)]
pub(crate) use dm_relay::{clear_test_marks, force_modal_mark, take_test_marks};
pub(crate) use do_dm_emit::emit_wrapped_do_dm_line;
#[cfg(test)]
pub use do_dm_filter::set_do_dm_stdout_mode;
pub use do_dm_filter::{enter_dm_only_stdout_mode, feed_do_dm_stdout_text};
mod stdout_mode;
#[cfg(test)]
pub(crate) use stdout_mode::DO_DM_TEST_LOCK;
pub use stdout_mode::{
    StdoutMode, StdoutModeGuard, do_dm_stdout_mode, enter_stdout_mode, heartbeat_stdout_suppressed,
    set_stdout_mode, stdout_display_discarded, stdout_mode,
};
pub(crate) use do_dm_on_emit::{deliver_relayed_direct_message, notify_completed_direct_messages};
pub use do_dm_on_emit::set_dm_on_emit_command;
#[cfg(test)]
pub(crate) use do_dm_on_emit::with_dm_on_emit;
pub use sentinel::{DM_END, DM_START, MALVIN_DONE, is_sentinel_line};
pub use stdout_display::{
    print_stdout_italic_text, print_stdout_line, print_stdout_raw_line,
    print_stdout_raw_line_with_ts, print_stdout_text,
};
pub use stdout_display_md::{
    flush_formatted_markdown_tables, print_stdout_line_with_markdown,
    print_stdout_text_with_markdown,
};

pub use acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, TermimadStdoutGate, acp_tee_display_line,
    acp_tee_log_line, format_line_acp_ansi_payload, print_stdout_acp_tee_line,
    print_stdout_acp_tee_line_with_timestamp, print_stdout_acp_tee_line_with_timestamp_dim_plain,
    print_stdout_acp_tool_summary_tee, termimad_inline_payload_for_stdout,
    termimad_text_lines_for_stdout,
};

#[cfg(test)]
mod acp_tee_termimad_tests;
#[cfg(test)]
mod acp_tee_tests;
#[cfg(test)]
mod format_tests;
#[cfg(test)]
#[path = "format_tests_b.rs"]
mod format_tests_b;
#[cfg(test)]
#[path = "output_kiss_cov_tests.rs"]
mod output_kiss_cov_tests;
#[cfg(test)]
mod stdout_log_tests;

pub(crate) use self::terminal_wrap::{
    stderr_line_wrap_meta, stdout_line_wrap_meta, wrap_words_bounded,
};

pub(crate) use stdout_terminal::print_stdout_display_line;
#[cfg(test)]
pub(crate) use stdout_terminal::{enable_stdout_capture, take_captured_stdout};

pub use who_tag::{
    LOG_TAG_INNER_WIDTH, WHO_A, WHO_B, WHO_H, WHO_M, WHO_O, WHO_P, WHO_R, WHO_T, WHO_U,
    format_acp_directional_tag_prefix, format_log_tag_inner, format_who_tag_delim,
    format_who_tag_prefix, is_command_prelude_line,
};

#[allow(unused_imports)]
pub(crate) use who_tag::{
    is_log_timestamp_token, payload_after_fixed_width_bracket_tag,
    payload_after_fixed_width_who_tag,
};

pub(crate) use crate::terminal_palette::{ANSI_DIM, ANSI_RESET};

pub(crate) use crate::time_format::timestamp_now_string;

pub use stdout_tee_env::{
    agent_stdout_tee_enabled, force_stdout_tee_from_env, stdout_is_interactive,
};

pub use crate::workspace::stdout_log_path::set_stdout_log_path;
pub(crate) use stdout_log_pair::{
    stdout_heartbeat_display_and_log_line, stdout_tagged_display_and_log_line,
};

pub use stderr_log::{print_log_error, print_log_warning, print_stderr_line};

pub use line_format::{format_line, format_line_with_timestamp};
#[cfg(test)]
pub(crate) use prompt_log::init_stdout_style_for_test;
pub use prompt_log::{
    append_outgoing_prompt_log_lines, init_stdout_style, print_outgoing_prompt_log,
};
pub use style::{ERROR_WHO, MALVIN_WHO, WARNING_WHO};
#[cfg(test)]
pub(crate) use style::{STDOUT_LOG_TEST_LOCK, push_captured_stderr_line};
pub(crate) use style::{
    append_stdout_log_line, log_use_color, stderr_use_color, stdout_use_color, who_tag_ansi,
};
#[cfg(test)]
pub use style::{clear_captured_stderr_lines, take_captured_stderr_lines};
