use super::render::{is_pipe_row, is_separator_line, render_table};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MdPiece {
    Source(String),
    Rendered(String),
}

#[derive(Default)]
pub(crate) struct MdTableStream {
    rows: Vec<String>,
    in_fence: bool,
}

impl MdTableStream {
    pub(crate) fn push(&mut self, line: &str, width: usize) -> Vec<MdPiece> {
        if self.in_fence {
            self.in_fence = !is_fence(line);
            return source_piece(line);
        }
        if is_fence(line) {
            self.in_fence = true;
            return self.finish_then_source(line, width);
        }
        if self.wants_row(line) {
            return self.buffer_row(line, width);
        }
        self.finish_then_source(line, width)
    }

    pub(crate) fn flush(&mut self, width: usize) -> Vec<MdPiece> {
        self.finish(width)
    }

    fn wants_row(&self, line: &str) -> bool {
        if self.rows.is_empty() {
            return is_pipe_row(line);
        }
        is_pipe_row(line) || is_separator_line(line)
    }

    fn buffer_row(&mut self, line: &str, width: usize) -> Vec<MdPiece> {
        if self.begins_another_table(line) {
            return self.rotate_header(line, width);
        }
        self.rows.push(line.to_string());
        Vec::new()
    }

    fn begins_another_table(&self, line: &str) -> bool {
        is_separator_line(line)
            && self.rows.iter().any(|row| is_separator_line(row))
            && self.rows.last().is_some_and(|row| !is_separator_line(row))
    }

    fn rotate_header(&mut self, line: &str, width: usize) -> Vec<MdPiece> {
        let header = self.rows.pop().unwrap_or_default();
        let out = self.finish(width);
        self.rows.push(header);
        self.rows.push(line.to_string());
        out
    }

    fn finish_then_source(&mut self, line: &str, width: usize) -> Vec<MdPiece> {
        let mut out = self.finish(width);
        out.extend(source_piece(line));
        out
    }

    fn finish(&mut self, width: usize) -> Vec<MdPiece> {
        let rows = std::mem::take(&mut self.rows);
        if let Some(rendered) = render_table(&rows, width) {
            return rendered.into_iter().map(MdPiece::Rendered).collect();
        }
        rows.into_iter().map(MdPiece::Source).collect()
    }
}

fn source_piece(line: &str) -> Vec<MdPiece> {
    vec![MdPiece::Source(line.to_string())]
}

fn is_fence(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

pub(crate) fn format_markdown_document(text: &str, width: usize) -> Vec<MdPiece> {
    let mut stream = MdTableStream::default();
    let mut out = Vec::new();
    for line in text.split_inclusive('\n') {
        let line = line.strip_suffix('\n').unwrap_or(line);
        out.extend(stream.push(line, width));
    }
    out.extend(stream.flush(width));
    out
}
