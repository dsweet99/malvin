//! External kiss witnesses for `code_flow::run_loop` private symbols.

#[test]
fn kiss_witness_code_gate_finish() {
    let _: Option<super::CodeGateFinish> = None;
    let _ = super::code_gate_outcome;
    let _ = super::emit_code_run_startup;
}
