#![allow(dead_code)]
#![allow(unused_imports)]

mod acp_code_fanout_mocks;
mod acp_code_run;
mod acp_core;
mod acp_do;
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
mod init_harness;
mod kpop_multiturn_support;
mod process;
#[cfg(unix)]
mod tidy_harness;
mod workspace;

pub use cli_parity_harness_run::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty_kpop::run_kpop_multiturn_investigate;

pub use acp_code_fanout_mocks::*;
pub use acp_code_run::*;
pub use acp_core::{acp_mock_js, chunk_line, *};
pub use acp_do::*;
pub use kpop_multiturn_support::*;
pub use process::{
    MALVIN_TEST_CMD_TIMEOUT, PlanSpawn, command_output_with_timeout, spawn_malvin_plan,
};
#[cfg(unix)]
pub use tidy_harness::{
    TidySpawn, bin_path_with_failing_gates, bin_path_with_fake_kiss,
    bin_path_with_kiss_fail_until_n_passes, spawn_tidy, spawn_tidy_with_learn,
    spawn_tidy_with_timeout, workspace_kiss_check_only,
};
pub use workspace::{
    only_run_dir, seed_git_kiss_cargo_gate_workspace, seed_malvin_checks, test_home_workspace,
    write_failing_gate_tools, write_fake_kiss, write_mock_executable,
};

#[cfg(unix)]
pub use do_stdout_harness::*;
#[cfg(unix)]
pub use do_stdout_harness_extra::*;
#[cfg(unix)]
pub use init_harness::*;

#[cfg(test)]
mod acp_mock_syntax_tests;
