use super::{ANSI_DIM, ANSI_RESET};
use termimad::MadSkin;

#[derive(Clone, Copy, Debug)]
pub struct TermimadStdoutGate {
    pub emit_stdout_markdown: bool,
    pub dim_payload: bool,
    pub allow_inline_styling: bool,
}

#[must_use]
pub fn termimad_inline_payload_for_stdout(line: &str, gate: TermimadStdoutGate) -> Option<String> {
    if !gate.emit_stdout_markdown || line.is_empty() || !gate.allow_inline_styling {
        return None;
    }
    let inner = termimad::inline(line).to_string();
    if gate.dim_payload {
        Some(dim_rendered_markup_payload(&inner))
    } else {
        Some(inner)
    }
}

#[must_use]
pub fn termimad_text_lines_for_stdout(
    line: &str,
    gate: TermimadStdoutGate,
    width: usize,
) -> Option<Vec<String>> {
    if !gate.emit_stdout_markdown || line.is_empty() || !gate.allow_inline_styling {
        return None;
    }
    if width < 3 || !needs_block_markdown_render(line) {
        return None;
    }
    Some(render_markdown_lines_for_stdout(line, gate, width))
}

fn render_markdown_lines_for_stdout(
    line: &str,
    gate: TermimadStdoutGate,
    width: usize,
) -> Vec<String> {
    let rendered = MadSkin::default()
        .text(line, Some(width.max(3)))
        .to_string();
    let mut lines: Vec<String> = rendered
        .trim_end_matches('\n')
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect();
    if gate.dim_payload {
        for rendered in &mut lines {
            *rendered = dim_rendered_markup_payload(rendered);
        }
    }
    lines
}

fn is_markdown_list_item(line: &str) -> bool {
    let trimmed = line.trim_start_matches(char::is_whitespace);
    matches!(trimmed.get(..2), Some("- " | "* " | "+ "))
}

fn is_markdown_ordered_list_item(line: &str) -> bool {
    let t = line.trim_start_matches(char::is_whitespace);
    let b = t.as_bytes();
    let mut i = 0usize;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && t.get(i..).is_some_and(|rest| rest.starts_with(". "))
}

fn is_markdown_heading(line: &str) -> bool {
    line.trim_start_matches(char::is_whitespace)
        .starts_with('#')
}

fn needs_block_markdown_render(line: &str) -> bool {
    is_markdown_heading(line) || is_markdown_list_item(line) || is_markdown_ordered_list_item(line)
}

fn dim_rendered_markup_payload(rendered: &str) -> String {
    let dimmed = rendered.replace(ANSI_RESET, &format!("{ANSI_RESET}{ANSI_DIM}"));
    format!("{ANSI_DIM}{dimmed}{ANSI_RESET}")
}

#[cfg(test)]
mod kiss_stringify_termimad {
    #[test]
    fn stringify_termimad_gate() {
        let _ = stringify!(super::TermimadStdoutGate);
        let _ = stringify!(super::termimad_inline_payload_for_stdout);
        let _ = stringify!(super::termimad_text_lines_for_stdout);
        let _ = stringify!(super::render_markdown_lines_for_stdout);
        let _ = stringify!(super::is_markdown_list_item);
        let _ = stringify!(super::is_markdown_ordered_list_item);
        let _ = stringify!(super::is_markdown_heading);
        let _ = stringify!(super::needs_block_markdown_render);
        let _ = stringify!(super::dim_rendered_markup_payload);
    }
}
