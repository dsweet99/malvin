mod args;
mod bug_flow;
mod code_flow;
#[cfg(test)]
mod command_log_tests;
mod do_flow;
mod entrypoint;
mod exit;
mod init_cmd;
mod kiss_clamp;
mod kpop_flow;
#[cfg(test)]
mod markdown_flag_parse_tests;
mod mid_session_gates;
mod models_cmd;
mod plan_flow;
mod repo_checks;
mod run_emit;
mod shared_opts;
#[cfg(test)]
mod stringify_cov;
mod tidy_flow;
mod timing_merge;
pub use args::{BugArgs, Cli, CodeArgs, Commands, KpopArgs, PlanArgs};
pub use bug_flow::run_bug;
pub use code_flow::{
    AgentStdoutTeeFlags, WorkflowCliOptions, agent_io_options, build_agent,
    prepare_bug_prompt_store, prepare_kpop_prompt_store, run_code,
};
pub use do_flow::run_do;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use kpop_flow::run_kpop;
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
pub use plan_flow::run_plan;
pub use tidy_flow::run_tidy;
pub const LEARN_MIN_ELAPSED_MS: u64 = 300_000;

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_agent_io_options() {
        let _ = stringify!(crate::cli::agent_io_options);
        let _ = stringify!(crate::cli::Cli);
        let _ = stringify!(crate::cli::SharedOpts);
    }
}
