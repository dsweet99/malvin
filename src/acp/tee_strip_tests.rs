// Test-only: trace tee strip contract (not used in non-test `malvin` builds).

/// Trace files may start with our `Command: …` prelude (see [`crate::acp::AcpSession::prompt`]). That
/// line is already printed once at CLI startup when tee is on; skip it here so tee does not repeat
/// the same invocation on stdout.
///
/// Post-hoc tee used to read the **whole** trace file before streaming existed; kept for unit tests
/// and the strip contract. Live tee skips the trace `Command:` line because startup already echoed it.
///
/// If the buffer begins with `Command: ` but contains **no** newline, the entire buffer is treated as
/// that single prelude line and this returns an empty slice (nothing left to tee after stripping).
pub(crate) fn strip_trace_invocation_line_for_tee(text: &str) -> &str {
    let Some(first) = text.lines().next() else {
        return text;
    };
    if !crate::output::is_command_prelude_line(first) {
        return text;
    }
    text.find('\n').map_or("", |i| &text[i + 1..])
}

#[test]
fn strips_malvin_command_prelude_only() {
    assert_eq!(
        strip_trace_invocation_line_for_tee("Command: malvin code x\n{\"a\":1}\n"),
        "{\"a\":1}\n"
    );
}

#[test]
fn leaves_json_only_trace_unchanged() {
    let j = "{\"jsonrpc\":\"2.0\"}\n";
    assert_eq!(strip_trace_invocation_line_for_tee(j), j);
}

/// Prelude only, no `\n`: entire buffer is the invocation line; strip yields nothing to print.
#[test]
fn single_line_command_prelude_without_newline_strips_to_empty() {
    assert_eq!(strip_trace_invocation_line_for_tee("Command: malvin code x"), "");
}

/// Same as [`single_line_command_prelude_without_newline_strips_to_empty`], with trailing newline
/// after the prelude only (no JSON body yet).
#[test]
fn command_prelude_line_only_then_newline_strips_to_empty() {
    assert_eq!(strip_trace_invocation_line_for_tee("Command: malvin code x\n"), "");
}
