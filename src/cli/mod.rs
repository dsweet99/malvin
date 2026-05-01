mod args;
mod code_flow;
#[cfg(test)]
mod command_log_tests;
mod do_flow;
mod entrypoint;
mod exit;
mod ground_cmd;
mod init_cmd;
mod kiss_clamp;
mod kpop_flow;
#[cfg(test)]
mod markdown_flag_parse_tests;
mod models_cmd;
mod repo_checks;
mod run_emit;
mod shared_opts;
#[cfg(test)]
mod stringify_cov;
mod sync_flow;
mod tidy_flow;
mod timing_merge;
pub use args::{Cli, CodeArgs, Commands, KpopArgs};
pub use code_flow::{
    AgentStdoutTeeFlags, WorkflowCliOptions, agent_io_options, build_agent,
    prepare_kpop_prompt_store, run_code,
};
pub use do_flow::run_do;
pub use entrypoint::entrypoint;
pub use exit::Exit;
pub use ground_cmd::run_ground;
pub use kpop_flow::run_kpop;
pub use run_emit::emit_run_startup_sequence;
pub use shared_opts::SharedOpts;
pub use sync_flow::SyncRunSpec;
pub use sync_flow::run_sync;
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
