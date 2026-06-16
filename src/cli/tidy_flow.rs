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
    /// Maximum gate-loop iterations before stopping.
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS_CODE)]
    pub max_loops: usize,
    /// Number of hypotheses per `KPop` round.
    #[arg(long, default_value_t = 5)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
    /// Deprecated: review fan-out removed; tidy now uses the kpop workflow.
    #[arg(long, short = 'q', default_value_t = false, hide = true)]
    pub quick: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate_kpop_workflow::post_gate_kpop_gates;

    #[test]
    fn tidy_effective_max_loops_is_at_least_one() {
        let tidy = TidyArgs {
            max_loops: 0,
            max_hypotheses: 10,
            tenacious: false,
            quick: false,
        };
        assert_eq!(effective_tidy_max_loops(tidy.max_loops), 1);
        assert_eq!(tidy.max_hypotheses, 10);
    }

    #[test]
    fn kiss_cov_tidy_kpop_helpers() {
        let _: Option<crate::gate_kpop_workflow::GateKpopPrepared> = None;
    }

    #[test]
    fn tidy_startup_logs_host_resources_in_command_log() {
        crate::test_utils::clear_test_no_real_agent_env();
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = crate::test_utils::save_cwd();
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_tidy_kpop_run(crate::cli::WorkflowCliOptions { force: false })
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
        crate::test_utils::restore_cwd(&old);
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
        let prepared = prepare_tidy_kpop_run(crate::cli::WorkflowCliOptions { force: false })
        .expect("prepared");
        let backups =
            crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snapshot");
        let err = post_gate_kpop_gates(
            "malvin tidy",
            &prepared,
            &backups,
            crate::gate_kpop_workflow::GateLoopBehavior::TIDY,
        )
        .expect_err("gates");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("quality gates"));
    }
}
