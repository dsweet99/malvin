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
        print_stdout_para(who, para, ts.as_str(), false);
    }
}

fn emit_ready_lines(lines: Vec<ReadyLine>) {
    for ready in lines {
        emit_ready(&ready.who, ready.piece);
    }
}

fn emit_ready(who: &str, piece: MdPiece) {
    let ts = super::timestamp_now_string();
    match piece {
        MdPiece::Rendered(line) => emit_tagged_rendered(who, &line, ts.as_str()),
        MdPiece::Source(line) => print_stdout_para(who, &line, ts.as_str(), true),
    }
}

fn print_stdout_para(who: &str, para: &str, ts: &str, emit_stdout_markdown: bool) {
    if emit_stdout_markdown && print_stdout_para_markdown(who, para, ts) {
        return;
    }
    let (max_payload, wrap) = stdout_line_wrap_meta(who, para);
    if !wrap {
        let (display, log) = super::stdout_tagged_display_and_log_line(who, para, Some(ts));
        print_stdout_rendered_line(&display, &log);
        return;
    }
    for seg in wrap_words_bounded(max_payload, para) {
        let (display, log) = super::stdout_tagged_display_and_log_line(who, &seg, Some(ts));
        print_stdout_rendered_line(&display, &log);
    }
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
            emit_tagged_rendered(who, &rendered, ts);
        }
        return true;
    }
    if !wrap {
        if let Some(rendered) = termimad_inline_payload_for_stdout(para, gate) {
            emit_tagged_rendered(who, &rendered, ts);
            return true;
        }
        return false;
    }
    let mut emitted = false;
    for seg in wrap_words_bounded(max_payload, para) {
        if let Some(rendered) = termimad_inline_payload_for_stdout(&seg, gate) {
            emit_tagged_rendered(who, &rendered, ts);
            emitted = true;
            continue;
        }
        let (display, log) = super::stdout_tagged_display_and_log_line(who, &seg, Some(ts));
        print_stdout_rendered_line(&display, &log);
        emitted = true;
    }
    emitted
}

fn emit_tagged_rendered(who: &str, rendered_payload: &str, ts: &str) {
    let (display, log) = super::stdout_tagged_display_and_log_line(who, rendered_payload, Some(ts));
    print_stdout_rendered_line(&display, &log);
}

#[cfg(test)]
mod tests {
    use super::{print_stdout_line_with_markdown, print_stdout_text_with_markdown};
    use crate::output::{
        STDOUT_LOG_TEST_LOCK, WHO_M, init_stdout_style_for_test, set_stdout_log_path,
    };

    fn capture_log(emit: bool, style: bool, line: &str) -> String {
        let _guard = STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        init_stdout_style_for_test(style);
        print_stdout_line_with_markdown(WHO_M, line, emit);
        set_stdout_log_path(None);
        std::fs::read_to_string(path).expect("read")
    }

    #[test]
    #[allow(unsafe_code)]
    fn rich_formats_bold_and_list_into_stdout_log_when_color_on() {
        let prev = std::env::var_os("NO_COLOR");
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        let bold = capture_log(true, true, "Branch is **57 commits** ahead.");
        let list = capture_log(true, true, "- **Removed** the mini path.");
        print_stdout_text_with_markdown(WHO_M, "plain\n", false);
        unsafe {
            match prev {
                Some(v) => std::env::set_var("NO_COLOR", v),
                None => std::env::remove_var("NO_COLOR"),
            }
        }
        assert!(bold.contains("57 commits") && !bold.contains("**57 commits**"));
        assert!(!bold.contains('\u{1b}'));
        assert!(list.contains("Removed") && !list.contains("**Removed**"));
    }

    #[test]
    fn keeps_markers_when_color_disabled() {
        let text = capture_log(true, false, "Branch is **57 commits** ahead.");
        assert!(text.contains("**57 commits**"));
    }
}
