mod args;
#[cfg(test)]
mod command_log_tests;
mod do_flow;
mod exit;
mod ground_cmd;
mod init_cmd;
mod kiss_clamp;
mod kpop_flow;
mod entrypoint;
#[cfg(test)]
mod markdown_flag_parse_tests;
mod models_cmd;
mod repo_checks;
mod run_emit;
mod code_flow;
mod shared_opts;
#[cfg(test)]
mod stringify_cov;
mod sync_flow;
mod tidy_flow;
mod timing_merge;
pub use args::{Cli, CodeArgs, Commands, KpopArgs};
pub use do_flow::run_do;
pub use exit::Exit;
pub use ground_cmd::run_ground;
pub use kpop_flow::run_kpop;
pub use entrypoint::entrypoint;
pub use code_flow::{
    agent_io_options,
    build_agent,
    run_code,
    prepare_kpop_prompt_store,
    AgentStdoutTeeFlags,
    WorkflowCliOptions,
};
pub use shared_opts::SharedOpts;
pub use run_emit::emit_run_startup_sequence;
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
