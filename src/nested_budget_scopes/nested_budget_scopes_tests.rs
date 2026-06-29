use super::BudgetScopeLayer;

#[test]
fn budget_scope_layer_all_has_seven_variants() {
    assert_eq!(BudgetScopeLayer::all().len(), 7);
}

#[test]
fn budget_scope_layer_metadata_contracts() {
    assert_eq!(
        BudgetScopeLayer::MiniHttpTurn.cli_flag(),
        Some("mini-max-http-turns")
    );
    assert!(!BudgetScopeLayer::MiniHttpTurn.respects_single_attempt());
    assert!(!BudgetScopeLayer::MiniHttpTurn.billing_fails_immediately());

    assert_eq!(
        BudgetScopeLayer::MiniTransportRetry.cli_flag(),
        None
    );
    assert!(BudgetScopeLayer::MiniTransportRetry.respects_single_attempt());
    assert!(BudgetScopeLayer::MiniTransportRetry.billing_fails_immediately());

    assert_eq!(
        BudgetScopeLayer::OuterKPopEngineLoop.cli_flag(),
        Some("max-loops")
    );
    assert_eq!(
        BudgetScopeLayer::AcpSpawnRetry.cli_flag(),
        Some("max-acp-retries")
    );
    assert!(BudgetScopeLayer::MiniGateIteration.respects_single_attempt());
    assert!(BudgetScopeLayer::MiniGateIteration.billing_fails_immediately());
}

#[test]
fn budget_scope_layer_variants_exist() {
    let _ = (
        BudgetScopeLayer::MiniTransportRetry,
        BudgetScopeLayer::MiniHttpTurn,
        BudgetScopeLayer::MiniBashExec,
        BudgetScopeLayer::MiniGateIteration,
        BudgetScopeLayer::MiniShrinkPass,
        BudgetScopeLayer::OuterKPopEngineLoop,
        BudgetScopeLayer::AcpSpawnRetry,
    );
}

#[test]
fn budget_scope_layer_all_layers_have_expected_cli_flags() {
    let expected = [
        (BudgetScopeLayer::MiniTransportRetry, None),
        (BudgetScopeLayer::MiniHttpTurn, Some("mini-max-http-turns")),
        (BudgetScopeLayer::MiniBashExec, Some("mini-max-bash-execs")),
        (BudgetScopeLayer::MiniGateIteration, Some("mini-max-gate-retries")),
        (BudgetScopeLayer::MiniShrinkPass, Some("mini-max-shrink-passes")),
        (BudgetScopeLayer::OuterKPopEngineLoop, Some("max-loops")),
        (BudgetScopeLayer::AcpSpawnRetry, Some("max-acp-retries")),
    ];
    assert_eq!(BudgetScopeLayer::all().len(), expected.len());
    for (layer, flag) in expected {
        assert_eq!(layer.cli_flag(), flag);
    }
}

#[test]
fn effective_max_attempts_single_attempt_forces_one_at_gate_layer() {
    assert_eq!(
        BudgetScopeLayer::MiniGateIteration.effective_max_attempts(5, true),
        1
    );
    assert_eq!(
        BudgetScopeLayer::MiniGateIteration.effective_max_attempts(5, false),
        5
    );
    assert_eq!(
        BudgetScopeLayer::MiniHttpTurn.effective_max_attempts(32, true),
        32
    );
}

#[test]
fn effective_outer_loop_iterations_is_at_least_one() {
    assert_eq!(BudgetScopeLayer::effective_outer_loop_iterations(0), 1);
    assert_eq!(BudgetScopeLayer::effective_outer_loop_iterations(3), 3);
}

#[test]
fn budget_scope_layer_single_attempt_and_billing_flags() {
    for layer in BudgetScopeLayer::all() {
        let single = layer.respects_single_attempt();
        let billing = layer.billing_fails_immediately();
        match layer {
            BudgetScopeLayer::MiniTransportRetry | BudgetScopeLayer::MiniGateIteration => {
                assert!(single);
                assert!(billing);
            }
            BudgetScopeLayer::AcpSpawnRetry => assert!(single),
            BudgetScopeLayer::MiniHttpTurn
            | BudgetScopeLayer::MiniBashExec
            | BudgetScopeLayer::MiniShrinkPass
            | BudgetScopeLayer::OuterKPopEngineLoop => {
                assert!(!single);
                assert!(!billing);
            }
        }
    }
}
