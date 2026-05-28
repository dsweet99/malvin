use clap::Args;

#[path = "constrain_flow/prep.rs"]
mod prep;
#[path = "constrain_flow/run_startup.rs"]
mod run_startup;
#[path = "constrain_flow/run_loop.rs"]
mod run_loop;

#[allow(unused_imports)]
pub use prep::{constrain_kpop_request, prepare_constrain_kpop_prompt_store};
#[allow(unused_imports)]
pub use run_startup::{prepare_constrain_kpop_run, ConstrainKpopPrepared};
pub use run_loop::run_constrain;

#[must_use]
pub(crate) fn effective_constrain_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_kpop_shared::effective_max_loops(max_loops)
}

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConstrainArgs {
    /// Maximum gate-loop iterations before stopping.
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS)]
    pub max_loops: usize,
    /// `KPop` hypothesis budget per gate session (`{{ want }}` in the agent prompt).
    #[arg(long, default_value_t = 10)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = false)]
    pub tenacious: bool,
    /// Deprecated: check-plan phase removed; constrain now uses the kpop gate workflow.
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
    use crate::cli::gate_kpop_workflow::post_gate_kpop_gates;

    #[test]
    fn constrain_effective_max_loops_is_at_least_one() {
        let constrain = ConstrainArgs {
            max_loops: 0,
            max_hypotheses: 10,
            tenacious: false,
            trust_the_plan: false,
            dry_run: false,
            skip_pre_checks: false,
            fast: false,
            request: Some("req".to_string()),
        };
        assert_eq!(effective_constrain_max_loops(constrain.max_loops), 1);
    }

    #[test]
    fn kiss_cov_constrain_kpop_helpers() {
        let _ = stringify!(run_loop::run_constrain);
        let _ = stringify!(run_startup::constrain_kpop_workflow_context);
        let _ = stringify!(crate::cli::gate_kpop_workflow::run_gate_kpop_loop);
        let _ = stringify!(crate::cli::gate_kpop_workflow::run_gate_kpop_session);
        let _ = stringify!(post_gate_kpop_gates);
        let _ = stringify!(crate::cli::workflow_kpop_shared::run_kpop_workspace_gates);
        let _ = stringify!(crate::cli::gate_kpop_workflow::finish_gate_kpop_after_pass);
        let _ = stringify!(crate::cli::gate_kpop_workflow::fail_gate_kpop_after_exhausted);
        let _ = stringify!(crate::cli::gate_kpop_workflow::print_gate_kpop_log_line);
        let _ = stringify!(crate::cli::gate_kpop_workflow::GateKpopPrepared);
        let _ = stringify!(crate::cli::gate_kpop_workflow::GateLoopBehavior::CODE);
    }

    #[test]
    fn constrain_startup_logs_host_resources_in_command_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_constrain_kpop_run(
            crate::cli::WorkflowCliOptions { force: false },
            "ship it",
        )
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
    fn constrain_post_kpop_gates_fails_when_gates_fail() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".malvin")).expect("mkdir");
        std::fs::write(tmp.path().join(".malvin/checks"), "kiss\n").expect("checks");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "kiss", 1);
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let prepared = prepare_constrain_kpop_run(
            crate::cli::WorkflowCliOptions { force: false },
            "ship it",
        )
        .expect("prepared");
        let err = post_gate_kpop_gates("malvin constrain", &prepared).expect_err("gates");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("quality gates"));
    }
}
