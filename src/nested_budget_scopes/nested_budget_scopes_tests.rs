use super::BudgetScopeLayer;

#[test]
fn budget_scope_layer_all_has_seven_variants() {
    assert_eq!(BudgetScopeLayer::all().len(), 7);
}

#[test]
fn budget_scope_layer_single_attempt_contracts() {
    assert!(!BudgetScopeLayer::MiniHttpTurn.respects_single_attempt());
    assert!(BudgetScopeLayer::MiniTransportRetry.respects_single_attempt());
    assert!(BudgetScopeLayer::MiniGateIteration.respects_single_attempt());
    assert!(BudgetScopeLayer::AcpSpawnRetry.respects_single_attempt());
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
fn budget_scope_layer_single_attempt_flags() {
    for layer in BudgetScopeLayer::all() {
        let single = layer.respects_single_attempt();
        match layer {
            BudgetScopeLayer::MiniTransportRetry
            | BudgetScopeLayer::MiniGateIteration
            | BudgetScopeLayer::AcpSpawnRetry => {
                assert!(single);
            }
            BudgetScopeLayer::MiniHttpTurn
            | BudgetScopeLayer::MiniBashExec
            | BudgetScopeLayer::MiniShrinkPass
            | BudgetScopeLayer::OuterKPopEngineLoop => {
                assert!(!single);
            }
        }
    }
}
