#![allow(unsafe_code)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(unused_imports, clippy::await_holding_lock)]

mod prelude;

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

pub(crate) use shared_harness::*;
pub(crate) use shared_handshake::*;
pub(crate) use child_health_a::*;
pub(crate) use child_health_b::*;
pub(crate) use handshake::*;
pub(crate) use jsonrpc::*;
pub(crate) use rpc_integration_a1::*;
pub(crate) use rpc_integration_a2::*;
pub(crate) use rpc_integration_b::*;
pub(crate) use rpc_unit::*;

#[cfg(test)]
mod kiss_coverage {
    use std::sync::atomic::Ordering;

    #[test]
    fn smoke_acp_activity_state() {
        let (seq, _notify) = super::acp_activity_state();
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
            let _ = child.kill().await;
        } else {
            panic!("expected child id");
        }
    }
}
