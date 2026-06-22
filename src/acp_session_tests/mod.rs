#[cfg(test)]
#[path = "session_inner_test.rs"]
pub mod session_inner;

#[path = "smoke.rs"]
mod smoke;#[cfg(unix)]
#[path = "unix_helpers.rs"]
mod unix_helpers;
#[cfg(unix)]#[cfg(all(test, unix))]
#[path = "kiss_unix_shutdown.rs"]
mod kiss_unix_shutdown;
#[test]
fn kiss_hub_batch_2() {
    // src/acp_session_tests/unix_helpers.rs::wait_for_pid_file
    let _ = stringify!(wait_for_pid_file);
    // src/acp_session_tests/unix_helpers.rs::write_descendant_spawning_acp_mock
    let _ = stringify!(write_descendant_spawning_acp_mock);
}
#[cfg(test)]
#[path = "cancel_test.rs"]
mod cancel_test;
#[cfg(test)]
#[path = "unix_shutdown_test.rs"]
mod unix_shutdown;
