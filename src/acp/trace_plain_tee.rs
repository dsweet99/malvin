use crate::output::{
    log_use_color, print_stdout_raw_line_with_ts, termimad_inline_payload_for_stdout,
    termimad_text_lines_for_stdout, TermimadStdoutGate,
};

pub(crate) fn print_tee_unprefixed_wrapped_line(line: &str, ts: &str) {
    let (max_payload, wrap) = crate::output::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        crate::output::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if !wrap {
        print_stdout_raw_line_with_ts(line, Some(ts));
        return;
    }
    for seg in crate::output::terminal_wrap::wrap_words_bounded(max_payload, line) {
        print_stdout_raw_line_with_ts(&seg, Some(ts));
    }
}

pub(crate) fn print_plain_tee_wrapped_line(line: &str, ts: &str, emit_stdout_markdown: bool) {
    if !emit_stdout_markdown {
        print_tee_unprefixed_wrapped_line(line, ts);
        return;
    }
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: log_use_color(),
    };
    let (max_payload, wrap) = crate::output::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        crate::output::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if let Some(rendered_lines) = termimad_text_lines_for_stdout(line, gate, max_payload) {
        for rendered in rendered_lines {
            print_stdout_raw_line_with_ts(&rendered, Some(ts));
        }
        return;
    }
    if !wrap {
        let rendered =
            termimad_inline_payload_for_stdout(line, gate).unwrap_or_else(|| line.to_string());
        print_stdout_raw_line_with_ts(&rendered, Some(ts));
        return;
    }
    for seg in crate::output::terminal_wrap::wrap_words_bounded(max_payload, line) {
        let rendered = termimad_inline_payload_for_stdout(&seg, gate).unwrap_or(seg);
        print_stdout_raw_line_with_ts(&rendered, Some(ts));
    }
}
