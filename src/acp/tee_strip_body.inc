// Skip duplicate `Command:` lines when teeing ACP trace files to stdout.

/// Trace files may start with our `Command: …` prelude (see [`crate::acp::AcpSession::prompt`]). That
/// line is already printed once at CLI startup when tee is on; skip it here so tee does not repeat
/// the same invocation on stdout.
///
/// Callers (for example [`maybe_tee_log`](crate::acp::maybe_tee_log)) read the **whole** trace file,
/// so this does not need to handle a prelude split across partial reads.
///
/// If the buffer begins with `Command: ` but contains **no** newline, the entire buffer is treated as
/// that single prelude line and this returns an empty slice (nothing left to tee after stripping).
pub(crate) fn strip_trace_invocation_line_for_tee(text: &str) -> &str {
    if !text.starts_with("Command: ") {
        return text;
    }
    text.find('\n').map_or("", |i| &text[i + 1..])
}
