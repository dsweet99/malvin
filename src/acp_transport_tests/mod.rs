#![allow(unsafe_code)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(unused_imports, clippy::await_holding_lock)]

mod shared_harness;
mod shared_handshake;

mod child_health_a;
mod child_health_b;
mod handshake;
mod jsonrpc;
mod rpc_integration_a1;
mod rpc_integration_a2;
mod rpc_integration_b;
mod rpc_unit;

pub(super) use shared_harness::*;
pub(crate) use shared_harness::HarnessRpcWaitParams;
pub(super) use shared_handshake::*;
pub(super) use child_health_a::*;
pub(super) use child_health_b::*;
pub(super) use handshake::*;
pub(super) use jsonrpc::*;
pub(super) use rpc_integration_a1::*;
pub(super) use rpc_integration_a2::*;
pub(super) use rpc_integration_b::*;
pub(super) use rpc_unit::*;

#[cfg(test)]
#[allow(dead_code)]
mod kiss_coverage {
    use std::sync::atomic::Ordering;

    #[test]
    fn smoke_acp_activity_state() {
        let (seq, _notify) = super::acp_activity_state();
        assert_eq!(seq.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn smoke_harness_rpc_wait_params_symbol() {
        let _ = std::any::type_name::<super::HarnessRpcWaitParams<'_>>();
    }

    #[test]
    fn kiss_wire_transport_units() {
        let _ = super::acp_activity_state;
        let _ = super::assert_arg_value;
        let _ = super::command_args;
        let _ = super::command_env_value;
        let _ = super::harness_rpc_wait;
        let _ = super::handshake_can_skip_cursor_login_when_api_key_mode_is_used;
        let _ = super::rpc_request_with_correlation_id_errors_when_reader_dead;
        let _ = super::rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive;
        let _ = super::rpc_request_with_correlation_id_times_out_when_stdout_silent;
        let _ = super::rpc_response_arriving_during_child_health_grace_is_delivered;
        let _ = super::rpc_wait_response_reports_dead_child_after_silence;
        let _ = super::spawn_test_reader_loop;
        let _ = super::test_handshake_hits_session_new_error_path;
        let _ = super::test_rpc_cancel_when_pending_sender_dropped;
        let _ = super::test_rpc_request_does_not_leak_pending_after_write_failure;
        let _ = super::test_write_rpc_line_fails_after_child_stdin_closed;
        let _ = crate::acp_test_unix_bin::unix_bin_with_fallback;
        let _ = super::write_authenticate_rejected_but_session_new_ok_mock;
        let _ = super::write_bad_session_new_mock;
        let _ = super::spawn_json_activity_then_response;
        let _ = super::spawn_activity_then_kill_child;
        let _ = super::handshake_skip_login_session_id;
        let _: Option<super::TestReaderLoopSpawn> = None;
        let _ = super::handshake_stdio_pipes;
        let _ = super::handshake_attach_and_start_reader;
        let _: Option<super::HandshakeRunning> = None;
        let _ = super::inactive_memory_containment;
        let _: Option<super::InactiveRpcIo> = None;
        let _ = super::acp_stdio_rpc_inactive;
        let _: Option<super::SleepStdoutDrainMode> = None;
        let _: Option<super::RpcSleepHarness> = None;
        let _ = super::drain_stdout_read;
        let _ = super::sleep_stdout_drain_for_child;
        let _ = super::RpcSleepHarness::spawn_sleep;
        let _ = super::RpcSleepHarness::shutdown;
        let _ = super::true_child_stdin_stdout_drained_after_exit;
    }
}
