use super::Exit;

#[cfg(test)]
use super::Commands;
pub(crate) use super::entrypoint_from;
#[cfg(test)]
use super::entrypoint_gates_only::{GatesOnlyDispatch, dispatch_gates_only_route};
#[cfg(test)]
pub(crate) use super::entrypoint_support::{
    dispatch_command, finish_entrypoint, prepare_cli_output, print_command_error, run_async_cli,
    try_tokio_runtime,
};
pub use entrypoint_from::entrypoint_from;

pub fn entrypoint() -> Exit {
    malvin::migrate_legacy_malvin_user_home();
    entrypoint_from(std::env::args_os())
}

#[cfg(test)]
#[path = "entrypoint_tenacious_tests.rs"]
mod entrypoint_tenacious_tests;

#[cfg(test)]
#[path = "entrypoint_doc_tests.rs"]
mod entrypoint_doc_tests;

#[cfg(test)]
#[path = "entrypoint_name_tests.rs"]
mod entrypoint_name_tests;

#[cfg(test)]
#[path = "entrypoint_name_unix_tests.rs"]
mod entrypoint_name_unix_tests;
