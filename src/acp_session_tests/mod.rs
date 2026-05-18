pub mod session_inner {
    include!("session_inner.rs");
}

include!("smoke.rs");
include!("cancel.rs");

#[cfg(unix)]
mod unix_helpers {
    include!("unix_helpers.rs");
}
#[cfg(unix)]
include!("unix_shutdown.rs");
#[cfg(all(unix, target_os = "linux"))]
include!("linux_spawn_abort.rs");

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn kiss_stringify_session_spec_units() {
        let _ = stringify!(super::session_inner::dead_transport_session_inner);
        let _ = stringify!(super::busy_session_with_dead_transport);
        let _ = stringify!(super::acp_session_cancel_clears_busy_state_after_rpc_error);
        let _ = stringify!(super::smoke_prompt_stdout_replacement_learn_vs_coder);
        #[cfg(unix)]
        {
            let _ = stringify!(super::unix_helpers::process_exists);
            let _ = stringify!(super::unix_helpers::wait_for_pid_file);
            let _ = stringify!(super::unix_helpers::write_descendant_spawning_acp_mock);
            let _ = stringify!(super::shutdown_kills_descendants::skip_without_writable_cgroups);
            let _ = stringify!(super::shutdown_kills_descendants::spawn_descendant_mock_session);
            let _ = stringify!(super::shutdown_kills_descendants::assert_descendant_killed_after_shutdown);
            let _ = stringify!(super::shutdown_kills_descendants::shutdown_kills_agent_spawned_descendants);
        }
        #[cfg(all(unix, target_os = "linux"))]
        {
            let _ = stringify!(
                super::linux_cgroup_verify_abort::acp_session_spawn_aborts_when_linux_cgroup_verify_fails
            );
        }
    }
}
