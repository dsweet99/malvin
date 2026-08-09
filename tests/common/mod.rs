#![allow(dead_code)]
#![allow(unused_imports)]

mod git_test_helpers;
mod child_wait;
mod cli_parity_harness_run;
#[cfg(unix)]
mod integration_cli_args;
#[cfg(unix)]
mod enn_hybrid_fixture;
#[cfg(unix)]
mod live_agent;
mod process;
mod contract;
pub mod observability_parity;
mod sandbox_test_helpers;
mod workspace;
#[cfg(unix)]
mod gate_bin_cache;

pub use cli_parity_harness_run::*;
#[cfg(unix)]
pub use contract::{fresh_workdir, prepend_fake_agent_models_to_path, sleep_child, write_peer_acp_lock};

pub use git_test_helpers::{git_commit_all, git_init};
pub use sandbox_test_helpers::{
    enable_test_fast_teardown, test_wait_until_async,
};
#[cfg(unix)]
pub use live_agent::{
    command_output_live_agent, live_agent_prereqs_met,
    require_openrouter_key_when_gate_set, LIVE_AGENT_CMD_TIMEOUT,
};
pub use process::{MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};
pub use gate_bin_cache::{static_fake_kiss_path_var, static_failing_gates_path_var, write_failing_gate_tools};
pub use workspace::{
    malvin_run_logs_bucket, only_run_dir, seed_fast_integration_malvin_config,
    seed_git_gate_workspace_cached, seed_git_kiss_cargo_gate_workspace, seed_malvin_checks,
    seed_malvin_config, test_home_workspace, fast_test_home_workspace, with_isolated_home,
    activate_test_home,
    write_fake_kiss, write_mock_executable, cached_mock_executable,
    seed_malvin_checks_legacy_fast,
};

#[cfg(unix)]
pub use integration_cli_args::INTEGRATION_TEST_MALVIN_ARGS;
#[cfg(unix)]
pub use enn_hybrid_fixture::*;
