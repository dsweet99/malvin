#![cfg(test)]

#[test]
fn kiss_stringify_session_units() {
    let _ = stringify!(super::prompt_stdout_replacement);
    let _ = stringify!(super::rpc_session_prompt_text);
    let _ = stringify!(super::do_split_trace_preamble);
}
