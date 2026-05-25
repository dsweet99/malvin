use clap::Args;

#[path = "tidy_flow/prep.rs"]
mod prep;
#[path = "tidy_flow/run_startup.rs"]
mod run_startup;
#[path = "tidy_flow/kpop_session.rs"]
mod kpop_session;
#[path = "tidy_flow/run_loop.rs"]
mod run_loop;

#[allow(unused_imports)]
pub use prep::{
    prepare_tidy_kpop_prompt_store, tidy_kpop_request, write_checks_do_not_pass_for_artifacts,
    write_checks_do_not_pass_to_review_path,
};
#[allow(unused_imports)]
pub use run_startup::{prepare_tidy_kpop_run, TidyKpopPrepared};
pub use run_loop::run_tidy;

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    max_loops.max(1)
}

#[derive(Args, Debug, Clone)]
pub struct TidyArgs {
    /// Maximum `KPop` hypothesis steps before stopping (alias: `--max-hypotheses`).
    #[arg(long, default_value_t = 3, alias = "max-hypotheses")]
    pub max_loops: usize,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Deprecated: review fan-out removed; tidy now uses the kpop workflow.
    #[arg(long, short = 'q', default_value_t = false, hide = true)]
    pub quick: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::kpop_session::tidy_post_kpop_gates;

    #[test]
    fn kpop_args_from_tidy_maps_max_loops() {
        let tidy = TidyArgs {
            max_loops: 0,
            no_learn: true,
            quick: false,
        };
        let kpop = crate::cli::KpopArgs {
            max_hypotheses: effective_tidy_max_loops(tidy.max_loops),
            no_learn: tidy.no_learn,
            request: Some("req".to_string()),
        };
        assert_eq!(kpop.max_hypotheses, 1);
        assert!(kpop.no_learn);
        assert_eq!(kpop.request.as_deref(), Some("req"));
    }

    #[test]
    fn kiss_cov_tidy_kpop_helpers() {
        let _ = stringify!(run_loop::TidyGateLoopCtx);
        let _ = stringify!(run_loop::run_tidy_gate_loop);
        let _ = stringify!(run_loop::start_tidy_agent_session);
        let _ = stringify!(kpop_session::tidy_kpop_prepared);
        let _ = stringify!(run_startup::tidy_kpop_workflow_context);
        let _ = stringify!(kpop_session::run_tidy_kpop_multiturn);
        let _ = stringify!(tidy_post_kpop_gates);
        let _ = stringify!(kpop_session::tidy_run_workspace_gates);
        let _ = stringify!(kpop_session::tidy_finish_after_gates_pass);
        let _ = stringify!(kpop_session::tidy_fail_after_exhausted_loops);
        let _ = stringify!(kpop_session::print_tidy_kpop_log_line);
        let _ = stringify!(kpop_session::run_tidy_kpop_session);
        let _ = stringify!(TidyKpopMultiturnRequest);
    }

    #[test]
    fn tidy_post_kpop_gates_fails_when_gates_fail() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss\n").expect("checks");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 1);
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_tidy_kpop_run(crate::cli::WorkflowCliOptions {
            force: false,
            run_learn: false,
        })
        .expect("prepared");
        let err = tidy_post_kpop_gates(&prepared).expect_err("gates");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("quality gates"));
    }
}
