use super::md_table::{MdPiece, ReadyLine, flush_stdout_markdown, push_stdout_markdown};
use super::stdout_display::print_stdout_rendered_line;
use super::stdout_line_wrap_meta;
use super::{
    TermimadStdoutGate, WHO_B, log_use_color, termimad_inline_payload_for_stdout,
    termimad_text_lines_for_stdout, wrap_words_bounded,
};

pub fn print_stdout_line_with_markdown(who: &str, line: &str, emit_stdout_markdown: bool) {
    if !emit_stdout_markdown {
        emit_ready_lines(flush_stdout_markdown());
        print_plain_parts(who, line);
        return;
    }
    let parts = paragraphs(line);
    let multiline = parts.len() > 1;
    for para in parts {
        append_raw_markdown_log(who, para);
        let (width, _) = stdout_line_wrap_meta(who, para);
        emit_ready_lines(push_stdout_markdown(who, para, width));
    }
    if multiline {
        emit_ready_lines(flush_stdout_markdown());
    }
}

pub fn print_stdout_text_with_markdown(who: &str, text: &str, emit_stdout_markdown: bool) {
    print_stdout_line_with_markdown(who, text, emit_stdout_markdown);
}

pub fn flush_formatted_markdown_tables() {
    emit_ready_lines(flush_stdout_markdown());
}

fn paragraphs(line: &str) -> Vec<&str> {
    let parts: Vec<&str> = super::logical_lines(line).collect();
    if parts.is_empty() { vec![""] } else { parts }
}

fn print_plain_parts(who: &str, line: &str) {
    for para in paragraphs(line) {
        let ts = super::timestamp_now_string();
        emit_para_segments(who, para, ts.as_str(), true);
    }
}

fn append_raw_markdown_log(who: &str, para: &str) {
    let ts = super::timestamp_now_string();
    super::append_stdout_log_line(&super::stdout_log_pair::tagged_log_line(&ts, who, para));
}

fn emit_ready_lines(lines: Vec<ReadyLine>) {
    for ready in lines {
        emit_ready(&ready.who, ready.piece);
    }
}

fn emit_ready(who: &str, piece: MdPiece) {
    let ts = super::timestamp_now_string();
    match piece {
        MdPiece::Rendered(line) => emit_one_segment(who, &line, ts.as_str(), false),
        MdPiece::Source(line) => {
            if !print_stdout_para_markdown(who, &line, ts.as_str()) {
                emit_para_segments(who, &line, ts.as_str(), false);
            }
        }
    }
}

fn emit_para_segments(who: &str, para: &str, ts: &str, record_log: bool) {
    let (max_payload, wrap) = stdout_line_wrap_meta(who, para);
    if !wrap {
        emit_one_segment(who, para, ts, record_log);
        return;
    }
    for seg in wrap_words_bounded(max_payload, para) {
        emit_one_segment(who, &seg, ts, record_log);
    }
}

fn emit_one_segment(who: &str, seg: &str, ts: &str, record_log: bool) {
    let (display, log) = super::stdout_tagged_display_and_log_line(who, seg, Some(ts));
    let log = if record_log { log } else { String::new() };
    print_stdout_rendered_line(&display, &log);
}

fn print_stdout_para_markdown(who: &str, para: &str, ts: &str) -> bool {
    let gate = TermimadStdoutGate {
        emit_stdout_markdown: true,
        dim_payload: who == WHO_B,
        allow_inline_styling: log_use_color(),
    };
    let (max_payload, wrap) = stdout_line_wrap_meta(who, para);
    if let Some(rendered_lines) = termimad_text_lines_for_stdout(para, gate, max_payload) {
        for rendered in rendered_lines {
            emit_one_segment(who, &rendered, ts, false);
        }
        return true;
    }
    if !wrap {
        if let Some(rendered) = termimad_inline_payload_for_stdout(para, gate) {
            emit_one_segment(who, &rendered, ts, false);
            return true;
        }
        return false;
    }
    let mut emitted = false;
    for seg in wrap_words_bounded(max_payload, para) {
        if let Some(rendered) = termimad_inline_payload_for_stdout(&seg, gate) {
            emit_one_segment(who, &rendered, ts, false);
            emitted = true;
            continue;
        }
        emit_one_segment(who, &seg, ts, false);
        emitted = true;
    }
    emitted
}

#[cfg(test)]
mod tests {
    use super::{print_stdout_line_with_markdown, print_stdout_text_with_markdown};
    use crate::ansi_strip::strip_ansi_escapes;
    use crate::output::{
        DO_DM_TEST_LOCK, STDOUT_LOG_TEST_LOCK, WHO_M, enable_stdout_capture,
        init_stdout_style_for_test, set_do_dm_stdout_mode, set_stdout_log_path,
        take_captured_stdout,
    };

    fn capture_log(emit: bool, style: bool, line: &str) -> (String, String) {
        let _dm = DO_DM_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _guard = STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        set_do_dm_stdout_mode(false);
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        init_stdout_style_for_test(style);
        enable_stdout_capture();
        print_stdout_line_with_markdown(WHO_M, line, emit);
        let display = take_captured_stdout();
        set_stdout_log_path(None);
        let log = std::fs::read_to_string(path).expect("read");
        (display, log)
    }

    #[test]
    #[allow(unsafe_code)]
    fn stdout_shows_termimad_and_log_keeps_raw_markdown() {
        let prev = std::env::var_os("NO_COLOR");
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        let (bold_display, bold_log) = capture_log(true, true, "Branch is **57 commits** ahead.");
        let (list_display, list_log) = capture_log(true, true, "- **Removed** the mini path.");
        print_stdout_text_with_markdown(WHO_M, "plain\n", false);
        unsafe {
            match prev {
                Some(v) => std::env::set_var("NO_COLOR", v),
                None => std::env::remove_var("NO_COLOR"),
            }
        }
        assert!(bold_log.contains("**57 commits**"), "{bold_log}");
        assert!(!bold_log.contains('\u{1b}'));
        assert!(list_log.contains("**Removed**"), "{list_log}");
        let bold_shown = strip_ansi_escapes(&bold_display);
        assert!(bold_shown.contains("57 commits") && !bold_shown.contains("**57 commits**"));
        let list_shown = strip_ansi_escapes(&list_display);
        assert!(list_shown.contains("Removed") && !list_shown.contains("**Removed**"));
    }

    #[test]
    fn keeps_markers_when_color_disabled() {
        let (_display, text) = capture_log(true, false, "Branch is **57 commits** ahead.");
        assert!(text.contains("**57 commits**"));
    }
}
