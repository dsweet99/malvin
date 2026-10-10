use std::num::NonZeroU32;

use crate::agent_process::AttemptCeiling;

use super::BudgetScopeLayer;
#[test]
fn budget_scope_layer_all_has_two_variants_and_budget_scope_layer_single_attempt_contracts() {
    {
        assert_eq!(BudgetScopeLayer::all().len(), 2);
    }
    {
        assert!(!BudgetScopeLayer::OuterRouterLoop.respects_single_attempt());
        assert!(BudgetScopeLayer::AcpAttempt.respects_single_attempt());
    }
}
#[test]
fn budget_scope_layer_variants_exist_and_effective_attempt_ceiling_single_attempt_forces_one_at_acp_layer()
 {
    {
        let _ = (
            BudgetScopeLayer::OuterRouterLoop,
            BudgetScopeLayer::AcpAttempt,
        );
    }
    {
        let five = NonZeroU32::new(5).expect("five");
        assert_eq!(
            BudgetScopeLayer::AcpAttempt.effective_attempt_ceiling(Some(five), true),
            AttemptCeiling::one()
        );
        assert_eq!(
            BudgetScopeLayer::AcpAttempt.effective_attempt_ceiling(Some(five), false),
            AttemptCeiling::AtMost(five)
        );
        assert_eq!(
            BudgetScopeLayer::AcpAttempt.effective_attempt_ceiling(None, false),
            AttemptCeiling::unlimited()
        );
        let thirty_two = NonZeroU32::new(32).expect("thirty two");
        assert_eq!(
            BudgetScopeLayer::OuterRouterLoop.effective_attempt_ceiling(Some(thirty_two), true),
            AttemptCeiling::AtMost(thirty_two)
        );
    }
}
#[test]
fn effective_outer_loop_iterations_is_at_least_one_and_budget_scope_layer_single_attempt_flags() {
    {
        assert_eq!(BudgetScopeLayer::effective_outer_loop_iterations(0), 1);
        assert_eq!(BudgetScopeLayer::effective_outer_loop_iterations(3), 3);
    }
    {
        for layer in BudgetScopeLayer::all() {
            let single = layer.respects_single_attempt();
            match layer {
                BudgetScopeLayer::AcpAttempt => {
                    assert!(single);
                }
                BudgetScopeLayer::OuterRouterLoop => {
                    assert!(!single);
                }
            }
        }
    }
}

#[test]
fn unlimited_ceiling_allows_every_attempt_count() {
    let ceiling = AttemptCeiling::unlimited();
    assert!(ceiling.allows_attempt(1));
    assert!(ceiling.allows_attempt(u32::MAX));
}
