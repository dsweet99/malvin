#![allow(dead_code)]
#![allow(unused_imports)]

mod acp_code_run;
mod acp_code_streaming;
mod acp_core;
mod acp_do;
#[cfg(unix)]
mod acp_tidy_interleaved;
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
#[cfg(unix)]
mod kiss_failing_gates;
mod kpop_multiturn_support;
mod process;
mod workspace;

pub use cli_parity_harness_run::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty_kpop::run_kpop_multiturn_investigate;

pub use acp_code_run::*;
pub use acp_code_streaming::*;
pub use acp_core::*;
pub use acp_do::*;
#[cfg(unix)]
pub use acp_tidy_interleaved::acp_mock_tidy_reviewer_lgtm_js;
#[cfg(unix)]
pub use kiss_failing_gates::write_failing_gate_tools;
pub use kpop_multiturn_support::*;
pub use process::{MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};
pub use workspace::{
    only_run_dir, seed_git_kiss_cargo_gate_workspace, test_home_workspace, write_fake_kiss,
    write_mock_executable,
};

#[cfg(unix)]
pub use do_stdout_harness::*;
#[cfg(unix)]
pub use do_stdout_harness_extra::*;
#[cfg(unix)]
pub use init_harness::*;
