#![allow(dead_code)]
#![allow(unused_imports)]

mod acp_code_run;
mod acp_code_streaming;
mod acp_core;
mod acp_do;
mod child_wait;
#[cfg(unix)]
mod do_stdout_harness;
#[cfg(unix)]
mod do_stdout_harness_extra;
#[cfg(unix)]
mod init_harness;
mod kpop_multiturn_support;
mod cli_parity_harness_run;
#[cfg(all(unix, target_os = "linux"))]
mod cli_parity_tty;
#[cfg(all(unix, target_os = "linux"))]
mod cli_parity_tty_kpop;
mod process;
mod workspace;

pub use cli_parity_harness_run::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty::*;
#[cfg(all(unix, target_os = "linux"))]
pub use cli_parity_tty_kpop::run_kpop_multiturn_investigate;

pub use acp_code_run::*;
pub use kpop_multiturn_support::*;
pub use acp_code_streaming::*;
pub use acp_core::*;
pub use acp_do::*;
pub use process::{command_output_with_timeout, MALVIN_TEST_CMD_TIMEOUT};
pub use workspace::{only_run_dir, test_home_workspace, write_fake_kiss, write_mock_executable};

#[cfg(unix)]
pub use do_stdout_harness::*;
#[cfg(unix)]
pub use do_stdout_harness_extra::*;
#[cfg(unix)]
pub use init_harness::*;
