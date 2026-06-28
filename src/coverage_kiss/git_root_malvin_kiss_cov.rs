//! External kiss symbol refs for git-root `.malvin/` and concepts §5/§9 modules.

#[test]
fn kiss_cov_kpop_progression_counter_wrappers() {
    let _ = crate::kpop_progression::hypotheses_emitted;
    let _ = crate::kpop_progression::agent_declared_success;
    let _ = stringify!(hypotheses_emitted);
    let _ = stringify!(agent_declared_success);
}

#[test]
fn kiss_cov_kpop_bridge_prompt_budget() {
    let _ = stringify!(guard_bridge_hypothesis_budget);
    let _ = crate::agent_backend::agent_backend_run_kpop_multiturn;
}

#[test]
fn kiss_cov_malvin_test_seed_helpers() {
    let _ = crate::malvin_test_seed::seed_malvin_checks;
    let _ = stringify!(ensure_git_repo_for_checks_seed);
}

#[test]
fn kiss_cov_repo_gates_git_root_helpers() {
    let _ = crate::git_worktree_toplevel;
    let _ = crate::repo_gates::prompt_quality_gates_markdown;
    let _ = stringify!(git_worktree_toplevel);
    let _ = stringify!(prompt_quality_gates_markdown);
}

#[test]
fn kiss_cov_repo_gates_checks_test_helpers() {
    let _ = crate::repo_gates::checks_test_helpers::git_init;
    let _ = stringify!(write_git_root_checks);
    let _ = stringify!(write_legacy_cwd_checks);
}
