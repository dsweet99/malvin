use super::{ANSI_DIM, ANSI_RESET};

#[derive(Clone, Copy, Debug)]
pub struct TermimadStdoutGate {
    pub emit_stdout_markdown: bool,
    pub dim_payload: bool,
    pub allow_inline_styling: bool,
}

#[must_use]
pub fn termimad_inline_payload_for_stdout(line: &str, gate: &TermimadStdoutGate) -> Option<String> {
    if !gate.emit_stdout_markdown || line.is_empty() || !gate.allow_inline_styling {
        return None;
    }
    let inner = termimad::inline(line).to_string();
    if gate.dim_payload {
        Some(format!("{ANSI_DIM}{inner}{ANSI_RESET}"))
    } else {
        Some(inner)
    }
}

#[cfg(test)]
mod kiss_stringify_termimad {
    #[test]
    fn stringify_termimad_gate() {
        let _ = stringify!(super::TermimadStdoutGate);
        let _ = stringify!(super::termimad_inline_payload_for_stdout);
    }
}
