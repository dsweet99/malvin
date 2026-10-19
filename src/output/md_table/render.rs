use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::fit::column_widths;

pub(crate) fn render_table(lines: &[String], max_width: usize) -> Option<Vec<String>> {
    let parsed = parse_table(lines)?;
    let widths = column_widths(&parsed.rows, max_width);
    Some(paint(&parsed.rows, &parsed.aligns, &widths))
}

fn paint(rows: &[Vec<String>], aligns: &[Align], widths: &[usize]) -> Vec<String> {
    let mut out = vec![rule(widths, '┌', '┬', '┐')];
    let mut body = rows.iter();
    if let Some(header) = body.next() {
        out.extend(paint_row(header, aligns, widths));
    }
    out.push(rule(widths, '├', '┼', '┤'));
    for row in body {
        out.extend(paint_row(row, aligns, widths));
    }
    out.push(rule(widths, '└', '┴', '┘'));
    out
}

fn paint_row(cells: &[String], aligns: &[Align], widths: &[usize]) -> Vec<String> {
    let wrapped: Vec<Vec<String>> = cells
        .iter()
        .enumerate()
        .map(|(idx, cell)| wrap_cell(cell, widths[idx]))
        .collect();
    let height = wrapped.iter().map(Vec::len).max().unwrap_or(1).max(1);
    (0..height)
        .map(|line_idx| paint_wrapped_line(&wrapped, aligns, widths, line_idx))
        .collect()
}

fn paint_wrapped_line(
    wrapped: &[Vec<String>],
    aligns: &[Align],
    widths: &[usize],
    line_idx: usize,
) -> String {
    let parts: Vec<String> = wrapped
        .iter()
        .enumerate()
        .map(|(idx, lines)| {
            let text = lines.get(line_idx).map_or("", String::as_str);
            pad(text, aligns[idx], widths[idx])
        })
        .collect();
    format!("│ {} │", parts.join(" │ "))
}

fn pad(text: &str, align: Align, width: usize) -> String {
    let extra = width.saturating_sub(unicode_width::UnicodeWidthStr::width(text));
    let (left, right) = pad_split(align, extra);
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

const fn pad_split(align: Align, extra: usize) -> (usize, usize) {
    match align {
        Align::Right => (extra, 0),
        Align::Center => (extra / 2, extra - extra / 2),
        Align::Just | Align::Left => (0, extra),
    }
}

fn rule(widths: &[usize], left: char, mid: char, right: char) -> String {
    let parts: Vec<String> = widths.iter().map(|width| "─".repeat(width + 2)).collect();
    format!("{left}{}{right}", parts.join(&mid.to_string()))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Align {
    Just,
    Left,
    Right,
    Center,
}

struct ParsedTable {
    aligns: Vec<Align>,
    rows: Vec<Vec<String>>,
}

pub(crate) fn is_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty()
        && trimmed.contains('|')
        && trimmed.contains('-')
        && trimmed
            .chars()
            .all(|ch| matches!(ch, '|' | ':' | '-' | ' '))
}

pub(crate) fn is_pipe_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.matches('|').count() >= 2
}

fn parse_table(lines: &[String]) -> Option<ParsedTable> {
    if !lines.iter().any(|line| is_separator_line(line)) {
        return None;
    }
    let mut aligns = Vec::new();
    let mut rows = Vec::new();
    for line in lines {
        if is_separator_line(line) {
            aligns = split_cells(line);
        } else {
            rows.push(split_cells(line));
        }
    }
    finish_parse(aligns, rows)
}

fn finish_parse(align_cells: Vec<String>, mut rows: Vec<Vec<String>>) -> Option<ParsedTable> {
    if rows.is_empty() {
        return None;
    }
    let cols = column_count(&rows, align_cells.len());
    if cols == 0 {
        return None;
    }
    let mut aligns: Vec<Align> = align_cells.iter().map(|cell| align_of(cell)).collect();
    while aligns.len() < cols {
        aligns.push(Align::Left);
    }
    aligns.truncate(cols);
    for row in &mut rows {
        while row.len() < cols {
            row.push(String::new());
        }
        row.truncate(cols);
    }
    Some(ParsedTable { aligns, rows })
}

fn column_count(rows: &[Vec<String>], align_len: usize) -> usize {
    rows.iter().map(Vec::len).max().unwrap_or(0).max(align_len)
}

fn align_of(cell: &str) -> Align {
    let cell = cell.trim();
    if cell.starts_with(':') && cell.ends_with(':') && cell.len() >= 2 {
        Align::Center
    } else if cell.ends_with(':') && cell.chars().any(|ch| ch != ':') {
        Align::Right
    } else {
        Align::Just
    }
}

fn split_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let without_lead = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = without_lead.strip_suffix('|').unwrap_or(without_lead);
    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn wrap_cell(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if UnicodeWidthStr::width(text) <= width {
        return vec![text.to_string()];
    }
    let chars: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        index = skip_wrap_indent(&chars, index, lines.is_empty());
        if index >= chars.len() {
            break;
        }
        let end = wrap_end(&chars, index, width);
        if end <= index {
            break;
        }
        lines.push(chars[index..end].iter().collect());
        index = end;
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn skip_wrap_indent(chars: &[char], index: usize, first: bool) -> usize {
    if first {
        return index;
    }
    let mut index = index;
    while index < chars.len() && chars[index] == ' ' {
        index += 1;
    }
    index
}

fn wrap_end(chars: &[char], start: usize, width: usize) -> usize {
    let mut vis = 0usize;
    let mut index = start;
    let mut last_space = None;
    while index < chars.len() && vis < width {
        let ch = chars[index];
        if ch == ' ' {
            last_space = Some(index);
        }
        vis += char_width(ch);
        index += 1;
    }
    if index >= chars.len() {
        return chars.len();
    }
    last_space.filter(|space| *space > start).unwrap_or(index)
}

fn char_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0)
}
