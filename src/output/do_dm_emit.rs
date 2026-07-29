//! Emit extracted `--do` DM bodies to process stdout.

pub(crate) fn emit_do_dm_body(body: &str) {
    if body.is_empty() {
        return;
    }
    for line in super::logical_lines(body) {
        emit_wrapped_do_dm_line(line);
    }
}

fn emit_wrapped_do_dm_line(line: &str) {
    if super::do_dm_mode::do_dm_stdout_markdown() {
        emit_markdown_do_dm_line(line);
        return;
    }
    emit_plain_wrapped_do_dm_line(line);
}

fn emit_plain_wrapped_do_dm_line(line: &str) {
    let (max_payload, wrap) = super::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        super::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if !wrap {
        super::stdout_terminal::emit_do_dm_body_line(line);
        return;
    }
    for seg in super::wrap_words_bounded(max_payload, line) {
        super::stdout_terminal::emit_do_dm_body_line(&seg);
    }
}

fn emit_markdown_do_dm_line(line: &str) {
    use super::{
        log_use_color, termimad_inline_payload_for_stdout, termimad_text_lines_for_stdout,
        TermimadStdoutGate,
    };
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: false,
        allow_inline_styling: log_use_color(),
    };
    let (max_payload, wrap) = super::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        super::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if let Some(rendered_lines) = termimad_text_lines_for_stdout(line, gate, max_payload) {
        for rendered in rendered_lines {
            super::stdout_terminal::emit_do_dm_body_line(&rendered);
        }
        return;
    }
    if !wrap {
        let rendered =
            termimad_inline_payload_for_stdout(line, gate).unwrap_or_else(|| line.to_string());
        super::stdout_terminal::emit_do_dm_body_line(&rendered);
        return;
    }
    for seg in super::wrap_words_bounded(max_payload, line) {
        let rendered = termimad_inline_payload_for_stdout(&seg, gate).unwrap_or(seg);
        super::stdout_terminal::emit_do_dm_body_line(&rendered);
    }
}
