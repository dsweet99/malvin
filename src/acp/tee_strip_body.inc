// Skip duplicate `Command:` lines when teeing ACP trace files to stdout.

/// Trace files may start with our `Command: …` prelude (see [`crate::acp::AcpSession::prompt`]). That
/// line is already printed once at CLI startup when tee is on; skip it here so tee does not repeat
/// the same invocation on stdout.
///
/// Post-hoc tee used to read the **whole** trace file before streaming existed; kept for unit tests
/// (`tee_strip_tests`) and the strip contract. Live tee skips the trace `Command:` line because
/// startup already echoed it.
///
/// If the buffer begins with `Command: ` but contains **no** newline, the entire buffer is treated as
/// that single prelude line and this returns an empty slice (nothing left to tee after stripping).
#[allow(dead_code)]
pub(crate) fn strip_trace_invocation_line_for_tee(text: &str) -> &str {
    if !text.starts_with("Command: ") {
        return text;
    }
    text.find('\n').map_or("", |i| &text[i + 1..])
}
