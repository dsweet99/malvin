use super::{ANSI_DIM, ANSI_RESET};
use crate::terminal_palette::ansi_tool_white;
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

pub(crate) fn agent_rendered_markup_payload(rendered: &str) -> String {
    let wrapped = rendered.replace(ANSI_RESET, &format!("{ANSI_RESET}{}", ansi_tool_white()));
    format!("{}{wrapped}{ANSI_RESET}", ansi_tool_white())
}

fn dim_rendered_markup_payload(rendered: &str) -> String {
    let dimmed = rendered.replace(ANSI_RESET, &format!("{ANSI_RESET}{ANSI_DIM}"));
    format!("{ANSI_DIM}{dimmed}{ANSI_RESET}")
}

#[cfg(test)]
mod termimad_tests {
    use super::{
        TermimadStdoutGate, dim_rendered_markup_payload, is_markdown_heading,
        is_markdown_list_item, is_markdown_ordered_list_item, needs_block_markdown_render,
        render_markdown_lines_for_stdout, termimad_inline_payload_for_stdout,
        termimad_text_lines_for_stdout,
    };

    #[test]
    fn termimad_gate_and_markdown_helpers() {
        assert!(is_markdown_heading("# h"));
        assert!(is_markdown_list_item("- x"));
        assert!(is_markdown_ordered_list_item("1. x"));
        assert!(needs_block_markdown_render("# h"));
        let gate = TermimadStdoutGate {
            emit_stdout_markdown: false,
            dim_payload: false,
            allow_inline_styling: false,
        };
        assert!(termimad_inline_payload_for_stdout("x", gate).is_none());
        assert!(termimad_text_lines_for_stdout("# h", gate, 80).is_none());
        let dimmed = dim_rendered_markup_payload("x");
        assert!(dimmed.contains(super::ANSI_DIM));
        let agent = super::agent_rendered_markup_payload("agent");
        assert!(agent.contains(super::ansi_tool_white()));
        let on = TermimadStdoutGate {
            emit_stdout_markdown: true,
            dim_payload: false,
            allow_inline_styling: true,
        };
        let lines = termimad_text_lines_for_stdout("# Title\n", on, 40);
        assert!(lines.is_some_and(|v| !v.is_empty()));
        let dim_gate = TermimadStdoutGate {
            emit_stdout_markdown: true,
            dim_payload: true,
            allow_inline_styling: true,
        };
        let rendered = render_markdown_lines_for_stdout("- item", dim_gate, 40);
        assert!(!rendered.is_empty());
        assert!(rendered[0].contains(super::ANSI_DIM));
    }
}
