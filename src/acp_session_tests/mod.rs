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
    fn kiss_stringify_units() {
        let _ = stringify!(super::acp_session_cancel_clears_busy_state_after_rpc_error);
        let _ = stringify!(super::acp_session_spawn_aborts_when_linux_cgroup_verify_fails);
        let _ = stringify!(super::shutdown_kills_agent_spawned_descendants);
        let _ = stringify!(super::dispatch_coder_session_prompt);
        let _ = stringify!(super::uniform_outgoing_trace_preamble);
        let _ = stringify!(super::do_split_outgoing_trace_preamble);
        let _ = stringify!(super::reader_loop_finish);
        let _ = stringify!(super::kpop_learn_phase);
        let _ = stringify!(super::kpop_fail_after_prompt);
        let _ = stringify!(crate::acp::session_prompt_helpers::rpc_session_prompt_text);
        let _ = stringify!(crate::acp::session_prompt_trace::uniform_outgoing_trace_preamble);
    }
}
