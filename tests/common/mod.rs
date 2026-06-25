#![allow(dead_code)]
#![allow(unused_imports)]

mod sandbox_test_helpers;
mod acp_code_fanout_mocks;
mod acp_code_run;
mod acp_core;
mod acp_do;
mod acp_do_dotfiles;
mod acp_tidy_kpop;
mod child_wait;
mod cli_parity_harness_run;
#[cfg(all(unix, target_os = "linux"))]
mod cli_parity_tty;
#[cfg(all(unix, target_os = "linux"))]
mod cli_parity_tty_kpop;
#[cfg(unix)]
mod do_stdout_harness;
#[cfg(unix)]
mod do_stdout_harness_extra;
#[cfg(unix)]
mod init_harness_run;
#[cfg(unix)]
mod init_harness;
#[cfg(unix)]
mod integration_cli_args;
#[cfg(unix)]
mod enn_hybrid_fixture;
mod kpop_multiturn_support;
mod kpop_outer_loop_support;
#[cfg(unix)]
mod live_agent;
mod process;
#[cfg(unix)]
mod code_harness;
#[cfg(unix)]
mod kpop_harness;
#[cfg(unix)]
mod tidy_harness;
#[cfg(unix)]
mod acp_delight_kpop;
#[cfg(unix)]
mod acp_explain_kpop;
#[cfg(unix)]
mod acp_revise_kpop;
#[cfg(unix)]
mod delight_harness;
#[cfg(unix)]
mod explain_harness;
#[cfg(unix)]
mod revise_harness;
mod contract;
mod workspace;

pub use cli_parity_harness_run::*;
#[cfg(unix)]
pub use contract::{fresh_workdir, sleep_child, write_peer_acp_lock};
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty_kpop::run_kpop_multiturn_investigate;

pub use sandbox_test_helpers::{
    enable_test_fast_teardown, test_wait_until_async,
};
pub use acp_code_fanout_mocks::*;
pub use acp_code_run::*;
pub use acp_core::{acp_mock_js, chunk_line, *};
pub use acp_do::*;
pub use acp_do_dotfiles::*;
pub use acp_tidy_kpop::*;
#[cfg(unix)]
pub use acp_delight_kpop::*;
#[cfg(unix)]
pub use acp_explain_kpop::*;
#[cfg(unix)]
pub use acp_revise_kpop::*;
#[cfg(unix)]
pub use delight_harness::*;
#[cfg(unix)]
pub use explain_harness::*;
#[cfg(unix)]
pub use revise_harness::*;
pub use kpop_multiturn_support::*;
pub use kpop_outer_loop_support::*;
#[cfg(unix)]
pub use live_agent::{
    command_output_live_agent, command_output_mini_live, live_agent_prereqs_met,
    LIVE_AGENT_CMD_TIMEOUT,
};
pub use process::{MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};
#[cfg(unix)]
pub use code_harness::{spawn_code, CodeSpawn};
#[cfg(unix)]
pub use kpop_harness::{spawn_kpop, KpopSpawn};
#[cfg(unix)]
pub use tidy_harness::{
    TidySpawn, bin_path_with_failing_gates, bin_path_with_fake_kiss,
    bin_path_with_kiss_fail_until_n_passes, spawn_tidy, spawn_tidy_with_timeout,
    workspace_kiss_check_only,
};
pub use workspace::{
    malvin_run_logs_bucket, only_run_dir, seed_git_kiss_cargo_gate_workspace, seed_malvin_checks,
    seed_malvin_config, test_home_workspace, with_isolated_home, write_failing_gate_tools,
    write_fake_kiss, write_mock_executable,
};

#[cfg(unix)]
pub use do_stdout_harness::*;
#[cfg(unix)]
pub use do_stdout_harness_extra::*;
#[cfg(unix)]
pub use init_harness::*;
#[cfg(unix)]
pub use integration_cli_args::{FAST_GATE_LOOP_TEST_ARGS, INTEGRATION_TEST_MALVIN_ARGS};
#[cfg(unix)]
pub use enn_hybrid_fixture::*;

#[cfg(test)]
mod acp_mock_syntax_tests;
