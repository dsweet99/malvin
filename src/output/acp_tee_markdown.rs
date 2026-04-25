#[derive(Clone, Copy, Debug)]
pub struct TermimadStdoutGate {
    pub emit_stdout_markdown: bool,
    pub dim_payload: bool,
    pub color_stdout: bool,
}

#[must_use]
pub fn termimad_inline_payload_for_stdout(line: &str, gate: &TermimadStdoutGate) -> Option<String> {
    if !gate.emit_stdout_markdown || gate.dim_payload || line.is_empty() || !gate.color_stdout {
        return None;
    }
    Some(termimad::inline(line).to_string())
}

#[cfg(test)]
mod kiss_stringify_termimad {
    #[test]
    fn stringify_termimad_gate() {
        let _ = stringify!(super::TermimadStdoutGate);
        let _ = stringify!(super::termimad_inline_payload_for_stdout);
    }
}
