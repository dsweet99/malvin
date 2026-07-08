//! External kiss witnesses for `router_flow` private symbols.

#[test]
fn kiss_witness_router_run_prep() {
    let _: Option<super::RouterRunPrep> = None;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(coder);
    let _ = stringify!(router_b_prompt);
    let _ = super::new_router_client;
    let _ = super::prepare_router_run;
}
