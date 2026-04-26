#[test]
fn kiss_stringify_acp_session_units() {
    let _ = stringify!(crate::acp::session::prompt_stdout_replacement);
    let _ = stringify!(crate::acp::session::rpc_session_prompt_text);
    let _ = stringify!(crate::acp::session::do_split_trace_preamble);
}
