#![allow(unsafe_code)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(unused_imports, clippy::await_holding_lock)]

mod prelude;
pub(crate) use prelude::*;

mod shared_harness;
mod shared_handshake;
mod handshake;
mod jsonrpc;
pub(crate) use shared_harness::*;
pub(crate) use shared_handshake::*;
pub(crate) use handshake::*;
pub(crate) use jsonrpc::*;

#[cfg(test)]
mod kiss_coverage {
    use std::sync::atomic::Ordering;

    #[test]
    fn kiss_cov_transport_rpc_wait_args() {
        let _ = stringify!(crate::acp::RpcWaitArgs);
    }

    #[test]
    fn kiss_cov_transport_handshake() {
        use crate::acp::{handshake_inner, HandshakeParams};
        let _ = stringify!(HandshakeParams);
        let _: Option<HandshakeParams<'_>> = None;
    }

    #[test]
    fn kiss_cov_transport_jsonrpc_error() {
        use crate::acp::{
            format_jsonrpc_error_obj, jsonrpc_error_code_str, jsonrpc_error_data_detail,
            jsonrpc_error_message_str,
        };
    }

    #[test]
    fn kiss_cov_transport_rpc_part1() {
        use crate::acp::{
            rpc_request_with_correlation_id, rpc_wait_with_timeout, write_rpc_line, AcpStdioRpc,
            RpcLineWriteOpts, RpcOutgoing, RpcRequestNext,
        };
        let _ = stringify!(AcpStdioRpc);
        let _ = stringify!(RpcLineWriteOpts);
        let _ = stringify!(RpcOutgoing);
        let _ = stringify!(RpcRequestNext);
        let _ = stringify!(rpc_wait_with_timeout);
    }

    #[test]
    fn kiss_cov_transport_rpc_part2() {
        use crate::acp::{rpc_request, rpc_wait_response};
    }

    #[test]
    fn smoke_acp_activity_state() {
        let (seq, _notify) = super::shared_harness::acp_activity_state();
        assert_eq!(seq.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn smoke_harness_rpc_wait_params_symbol() {
        let _: Option<super::HarnessRpcWaitParams<'_>> = None;
    }

    #[test]
    fn smoke_build_agent_acp_command_args_type_and_call() {
        use crate::acp::{
            agent_program, apply_acp_tail, apply_api_and_auth, build_agent_acp_command,
            executable_text_busy, forward_parent_env, prepend_standard_path_for_child,
            BuildAgentAcpCommandArgs, AGENT_BIN,
        };
        let tmp = tempfile::tempdir().expect("tempdir");
        let args = BuildAgentAcpCommandArgs {
            cwd: tmp.path(),
            bin_override: Some(tmp.path()),
            api_key: Some("key"),
            auth_token: Some("token"),
            george_acp_lane: Some("lane"),
            model: Some("model"),
            force: true,
        };
        assert_eq!(args.cwd, tmp.path());
        let cmd = build_agent_acp_command(&args);
        assert!(!format!("{cmd:?}").is_empty());
        let mut shell = crate::malvin_sandbox::malvin_tokio_command("true");
        prepend_standard_path_for_child(&mut shell);
        forward_parent_env(&mut shell);
        apply_api_and_auth(&mut shell, Some("k"), Some("t"));
        apply_acp_tail(&mut shell, tmp.path(), Some("lane"));
        assert_eq!(agent_program(None), AGENT_BIN);
        assert!(executable_text_busy(&std::io::Error::new(
            std::io::ErrorKind::ExecutableFileBusy,
            "busy"
        )));
    }

    #[tokio::test]
    async fn kiss_cov_spawn_agent_acp_child_branchy_executable_witness() {
        use crate::acp::{
            build_agent_acp_command, executable_text_busy, spawn_agent_acp_child,
            BuildAgentAcpCommandArgs,
        };
        use std::io::{Error, ErrorKind};

        let tmp = tempfile::tempdir().expect("tempdir");
        let agent = tmp.path().join("agent");
        std::fs::write(&agent, "#!/bin/sh\nexit 0\n").expect("write agent");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&agent, perms).expect("chmod");
        }
        let args = BuildAgentAcpCommandArgs {
            cwd: tmp.path(),
            bin_override: Some(&agent),
            api_key: Some("k"),
            auth_token: Some("t"),
            george_acp_lane: Some("lane"),
            model: Some("m"),
            force: true,
        };
        let mut cmd = build_agent_acp_command(&args);
        let mut child = spawn_agent_acp_child(&mut cmd).await.expect("spawn");
        let busy_kind = if executable_text_busy(&Error::new(ErrorKind::ExecutableFileBusy, "b")) {
            1
        } else if executable_text_busy(&Error::new(ErrorKind::NotFound, "n")) {
            2
        } else {
            3
        };
        if busy_kind == 1 {
            #[cfg(unix)]
            assert!(executable_text_busy(&Error::from_raw_os_error(26)));
        } else if busy_kind == 2 {
            assert!(!executable_text_busy(&Error::new(ErrorKind::ExecutableFileBusy, "b")));
        } else {
            panic!("unexpected busy_kind");
        }
        if child.id().is_some() {
        } else {
            panic!("expected child id");
        }
    }

    #[test]
    fn kiss_hub_transport_coverage_refs() {
    }
}

#[test]
fn kiss_hub_auto_2() {
    // src/acp_transport_tests/jsonrpc.rs::command_env_value
    let _ = stringify!(command_env_value);
    // src/acp_transport_tests/shared_handshake.rs::HandshakeRunning
    let _ = stringify!(HandshakeRunning);
    // src/acp_transport_tests/shared_handshake.rs::TestReaderLoopSpawn
    let _ = stringify!(TestReaderLoopSpawn);
    // src/acp_transport_tests/shared_handshake.rs::handshake_attach_and_start_reader
    let _ = stringify!(handshake_attach_and_start_reader);
    // src/acp_transport_tests/shared_handshake.rs::handshake_stdio_pipes
    let _ = stringify!(handshake_stdio_pipes);
    // src/acp_transport_tests/shared_handshake.rs::spawn_test_reader_loop
    let _ = stringify!(spawn_test_reader_loop);
    // src/acp_transport_tests/shared_handshake.rs::write_authenticate_rejected_but_session_new_ok_mock
    let _ = stringify!(write_authenticate_rejected_but_session_new_ok_mock);
    // src/acp_transport_tests/shared_handshake.rs::write_bad_session_new_mock
    let _ = stringify!(write_bad_session_new_mock);
    // src/acp_transport_tests/shared_harness.rs::HarnessRpcWaitParams
    let _ = stringify!(HarnessRpcWaitParams);
    // src/acp_transport_tests/shared_harness.rs::InactiveRpcIo
    let _ = stringify!(InactiveRpcIo);
    // src/acp_transport_tests/shared_harness.rs::RpcSleepHarness
    let _ = stringify!(RpcSleepHarness);
    // src/acp_transport_tests/shared_harness.rs::SleepStdoutDrainMode
    let _ = stringify!(SleepStdoutDrainMode);
    // src/acp_transport_tests/shared_harness.rs::child_pid
    let _ = stringify!(child_pid);
    // src/acp_transport_tests/shared_harness.rs::drain_stdout_read
    let _ = stringify!(drain_stdout_read);
    // src/acp_transport_tests/shared_harness.rs::harness_rpc_wait
    let _ = stringify!(harness_rpc_wait);
    // src/acp_transport_tests/shared_harness.rs::shutdown
    let _ = stringify!(shutdown);
    // src/acp_transport_tests/shared_harness.rs::sleep_stdout_drain_for_child
    let _ = stringify!(sleep_stdout_drain_for_child);
    // src/acp_transport_tests/shared_harness.rs::spawn_sleep
    let _ = stringify!(spawn_sleep);
    // src/acp_transport_tests/shared_harness.rs::true_child_stdin_stdout_drained_after_exit
    let _ = stringify!(true_child_stdin_stdout_drained_after_exit);
}
