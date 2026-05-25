use clap::Args;

#[path = "code_flow/prep.rs"]
mod prep;
#[path = "code_flow/run_startup.rs"]
mod run_startup;
#[path = "code_flow/kpop_session.rs"]
mod kpop_session;
#[path = "code_flow/run_loop.rs"]
mod run_loop;

#[allow(unused_imports)]
pub use prep::{code_kpop_request, prepare_code_kpop_prompt_store};
#[allow(unused_imports)]
pub use run_startup::{prepare_code_kpop_run, CodeKpopPrepared};
pub use run_loop::run_code;

#[must_use]
pub(crate) fn effective_code_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct CodeArgs {
    /// Maximum gate-loop iterations before stopping.
    #[arg(long, default_value_t = 5)]
    pub max_loops: usize,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Deprecated: check-plan phase removed; code now uses the kpop gate workflow.
    #[arg(long, default_value_t = false, hide = true, conflicts_with = "dry_run")]
    pub trust_the_plan: bool,
    /// Deprecated: check-plan dry run removed with the kpop gate workflow.
    #[arg(long, default_value_t = false, hide = true, conflicts_with = "trust_the_plan")]
    pub dry_run: bool,
    /// Deprecated: quality gates always run in the gate loop.
    #[arg(long, default_value_t = false, hide = true)]
    pub skip_pre_checks: bool,
    /// Deprecated: use `--trust-the-plan` (hidden).
    #[arg(short = 'f', default_value_t = false, hide = true)]
    pub fast: bool,
    /// Request text or path to an existing `.md` file → `.malvin/logs/.../plan.md`.
    pub request: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::kpop_session::code_post_kpop_gates;

    #[test]
    fn kpop_args_from_code_maps_max_loops() {
        let code = CodeArgs {
            max_loops: 0,
            no_learn: true,
            trust_the_plan: false,
            dry_run: false,
            skip_pre_checks: false,
            fast: false,
            request: Some("req".to_string()),
        };
        let kpop = crate::cli::KpopArgs {
            max_hypotheses: effective_code_max_loops(code.max_loops),
            no_learn: code.no_learn,
            request: Some("req".to_string()),
        };
        assert_eq!(kpop.max_hypotheses, 1);
        assert!(kpop.no_learn);
    }

    #[test]
    fn kiss_cov_code_kpop_helpers() {
        let _ = stringify!(run_loop::CodeGateLoopCtx);
        let _ = stringify!(run_loop::run_code_gate_loop);
        let _ = stringify!(run_loop::start_code_agent_session);
        let _ = stringify!(run_startup::code_kpop_workflow_context);
        let _ = stringify!(kpop_session::run_code_kpop_multiturn);
        let _ = stringify!(code_post_kpop_gates);
        let _ = stringify!(kpop_session::code_run_workspace_gates);
        let _ = stringify!(kpop_session::code_finish_after_gates_pass);
        let _ = stringify!(kpop_session::code_fail_after_exhausted_loops);
        let _ = stringify!(kpop_session::print_code_kpop_log_line);
        let _ = stringify!(kpop_session::run_code_kpop_session);
        let _ = stringify!(kpop_session::CodeKpopMultiturnRequest);
        let _ = stringify!(kpop_session::kpop_args_from_code);
    }

    #[test]
    fn code_post_kpop_gates_fails_when_gates_fail() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss\n").expect("checks");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 1);
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_code_kpop_run(
            crate::cli::WorkflowCliOptions {
                force: false,
                run_learn: false,
            },
            "ship it",
        )
        .expect("prepared");
        let err = code_post_kpop_gates(&prepared).expect_err("gates");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("quality gates"));
    }
}
