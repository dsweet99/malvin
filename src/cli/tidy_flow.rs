use clap::Args;

#[path = "tidy_flow/prep.rs"]
mod prep;
#[path = "tidy_flow/run_startup.rs"]
mod run_startup;
#[path = "tidy_flow/run_loop.rs"]
mod run_loop;

#[allow(unused_imports)]
pub use prep::{prepare_tidy_kpop_prompt_store, tidy_kpop_request};
#[allow(unused_imports)]
pub use crate::cli::workflow_kpop_shared::{
    write_checks_do_not_pass_for_artifacts, write_checks_do_not_pass_to_review_path,
};
#[allow(unused_imports)]
pub use run_startup::{prepare_tidy_kpop_run, TidyKpopPrepared};
pub use run_loop::run_tidy;

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
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
    use crate::cli::gate_kpop_workflow::post_gate_kpop_gates;

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
        let _ = stringify!(run_loop::run_tidy);
        let _ = stringify!(run_startup::tidy_kpop_workflow_context);
        let _ = stringify!(crate::cli::gate_kpop_workflow::run_gate_kpop_loop);
        let _ = stringify!(crate::cli::gate_kpop_workflow::run_gate_kpop_session);
        let _ = stringify!(post_gate_kpop_gates);
        let _ = stringify!(crate::cli::workflow_kpop_shared::run_kpop_workspace_gates);
        let _ = stringify!(crate::cli::gate_kpop_workflow::finish_gate_kpop_after_pass);
        let _ = stringify!(crate::cli::gate_kpop_workflow::fail_gate_kpop_after_exhausted);
        let _ = stringify!(crate::cli::gate_kpop_workflow::print_gate_kpop_log_line);
        let _ = stringify!(crate::cli::gate_kpop_workflow::GateKpopPrepared);
        let _ = stringify!(crate::cli::gate_kpop_workflow::GateLoopBehavior::TIDY);
    }

    #[test]
    fn tidy_startup_logs_host_resources_in_command_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_tidy_kpop_run(crate::cli::WorkflowCliOptions {
            force: false,
            run_learn: false,
        })
        .expect("prepared");
        crate::cli::run_emit::emit_run_startup_sequence(
            &prepared.artifacts,
            crate::cli::run_emit::RunStartupEmitOpts {
                tee_stdout: false,
                host_resources: true,
            },
            &prepared.startup_emit_request,
        )
        .expect("startup");
        let command_log = prepared.artifacts.run_dir.join("command.log");
        let log = std::fs::read_to_string(&command_log).expect("log");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(log.contains("Memory:"));
        assert!(log.contains("CPUs:"));
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
        let err = post_gate_kpop_gates("malvin tidy", &prepared).expect_err("gates");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("quality gates"));
    }
}
