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
