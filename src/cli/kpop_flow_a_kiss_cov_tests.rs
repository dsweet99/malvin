//! External kiss witnesses for `kpop_flow_a` types and test helpers.

#[test]
fn kiss_witness_kpop_flow_a_types() {
    let _: Option<super::KpopPrepared> = None;
    let _: Option<super::KpopArtifactsEarly> = None;
    let _: Option<super::KpopAcpMultiturnCtx> = None;
    let _ = super::prepare_kpop_artifacts;
    let _ = super::finish_kpop_prepared;
    let _ = super::kpop_boot_store_client_prepared;
}

#[test]
fn kiss_witness_kpop_flow_a_tests() {
    let _ = stringify!(seed_short_id_lookup_fixture);
    let _ = stringify!(seed_kpop_multiturn_mock_workspace);
    let _ = stringify!(run_kpop_multiturn_mock_once);
}
