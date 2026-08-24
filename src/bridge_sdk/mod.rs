mod drain_idle;
mod log_adapter;
mod log_adapter_tool;
mod session;
mod session_handshake;
mod session_io;
mod stream_log;
mod timing;

#[cfg(test)]
#[path = "drain_idle_tests.rs"]
mod drain_idle_tests;

#[cfg(test)]
#[path = "drain_idle_policy_tests.rs"]
mod drain_idle_policy_tests;

#[cfg(test)]
pub(crate) use drain_idle::{DrainHealthVerdict, await_next_with_idle_using};
pub(crate) use drain_idle::{
    DrainIdleHealthCtx, DrainIdleLabels, DrainIdleTurn, await_next_with_idle_in_turn,
};
#[cfg(test)]
pub(crate) use drain_idle::DrainIdleClock;

pub(crate) use log_adapter::{feed_do_dm_run_result, handle_stream_event};
pub use session::{BridgeSession, BridgeSpawnArgs, BridgeWire, SDK_BRIDGE_MAX_AGE, ToolCallStart};
pub use stream_log::StreamLog;
pub use session_io::write_request;
pub(crate) use session_io::{
    CreateArgs, ResumeArgs, run_done_status_is_failure, send_create, send_resume, start_mem_watch,
};
pub use timing::{note_sdk_step, record_sdk_usage};

#[cfg(test)]
mod protocol_reexport_tests {
    #[test]
    fn bridge_protocol_is_shared() {
        let _ = crate::bridge_protocol::encode_request;
        let _ = crate::bridge_protocol::decode_event;
        let _ = super::write_request;
        let _ = stringify!(BridgeSession);
        let _ = stringify!(BridgeSpawnArgs);
        let _ = stringify!(BridgeWire);
        let _ = stringify!(NodeBridge);
        let _ = stringify!(ToolCallStart);
        let _ = stringify!(SDK_BRIDGE_MAX_AGE);
        let _ = stringify!(send_create);
        let _ = stringify!(send_resume);
        let _ = stringify!(start_mem_watch);
        let _ = stringify!(note_sdk_step);
        let _ = stringify!(record_sdk_usage);
        let _ = stringify!(logged_run_done);
        let _ = stringify!(canonicalize_run_done);
    }
}
