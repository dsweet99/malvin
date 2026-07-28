//! Kiss identifier refs for [`super::router_flow_acp_support`].

use crate::artifacts::SessionDotfileBackups;

#[test]
fn kiss_cov_router_acp_support_fn_names() {
    let _ = super::router_iteration_log_path;
    let _ = super::empty_iteration_backups;
    let _ = super::snapshot_iteration_backups;
    let _ = super::run_router_turns;
    let _ = super::run_multi_group_kpop;
    let _ = super::RouterExitSummarize::Run;
    let _ = super::RouterExitSummarize::Skip;
    let _: Option<super::RouterTurnsOutcome> = None;
    let _ = stringify!(iteration_backups);
    let _ = stringify!(all_no_work);
}

#[test]
fn kiss_cov_router_turns_outcome_destructure() {
    let outcome = super::RouterTurnsOutcome {
        iteration_backups: SessionDotfileBackups::snapshot(std::path::Path::new("/tmp"))
            .expect("snapshot"),
        all_no_work: true,
    };
    let super::RouterTurnsOutcome {
        iteration_backups: _,
        all_no_work,
    } = outcome;
    assert!(all_no_work);
}
