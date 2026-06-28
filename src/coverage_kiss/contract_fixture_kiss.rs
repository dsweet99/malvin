//! External kiss witnesses for `contract_fixture.rs`.

#[test]
fn kiss_cov_contract_fixture_symbols() {
    let _ = crate::acp::open_contract_trace_writer;
    let _ = crate::acp::tee_coalesced_tool_execute;
    let _ = crate::acp::contract_acp_tee_tool_fixture;
}
