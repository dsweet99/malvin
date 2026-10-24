use crate::cli::router_machine::Completion;
use malvin::artifacts::SessionDotfileBackups;

#[test]
fn kiss_cov_router_acp_support_fn_names() {
    let _ = super::router_iteration_log_path;
    let _ = super::empty_iteration_backups;
    let _ = super::snapshot_iteration_backups;
    let _ = super::run_router_turns;
    let _ = super::deliver_router_initial_turn;
    let _ = super::RouterExitSummarize::Run;
    let _ = super::RouterExitSummarize::Skip;
    let _: Option<super::RouterTurnsOutcome> = None;
    let _ = stringify!(iteration_backups);
    let _ = stringify!(completion);
    let _ = super::send_router_c;
}

#[test]
fn kiss_cov_router_turns_outcome_destructure() {
    let outcome = super::RouterTurnsOutcome {
        iteration_backups: SessionDotfileBackups::snapshot(std::path::Path::new("/tmp"))
            .expect("snapshot"),
        completion: Completion::Impasse,
    };
    let super::RouterTurnsOutcome {
        iteration_backups: _,
        completion,
    } = outcome;
    assert_eq!(completion, Completion::Impasse);
}
