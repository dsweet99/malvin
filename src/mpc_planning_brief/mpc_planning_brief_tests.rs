use super::MpcPlanningBriefAspect;

#[test]
fn mpc_planning_brief_aspect_all_has_six_variants() {
    assert_eq!(MpcPlanningBriefAspect::all().len(), 6);
}

#[test]
fn mpc_planning_brief_aspect_primary_modules_are_unique_and_non_empty() {
    let mut seen = std::collections::HashSet::new();
    for aspect in MpcPlanningBriefAspect::all() {
        let module = aspect.primary_module();
        assert!(!module.is_empty(), "empty primary_module for {aspect:?}");
        assert!(seen.insert(module), "duplicate primary_module {module} for {aspect:?}");
    }
}

#[test]
fn all_aspects_have_runtime_references() {
    let sources = [
        include_str!("../kpop_engine/mpc_planner.rs"),
        include_str!("../kpop_engine/run_loop_exit.rs"),
        include_str!("../kpop_engine/run_loop.rs"),
    ];
    for aspect in MpcPlanningBriefAspect::all() {
        let needle = format!("MpcPlanningBriefAspect::{aspect:?}");
        assert!(
            sources.iter().any(|src| src.contains(&needle)),
            "missing production reference for {aspect:?}"
        );
    }
}
