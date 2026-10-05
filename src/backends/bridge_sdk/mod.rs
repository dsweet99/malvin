mod child_process_session;
mod child_stderr;
mod drain_idle;
mod json_line_session;
mod turn_protocol;
mod log_adapter;
mod log_adapter_tool;
mod session;
mod session_handshake;
mod session_io;
#[path = "session_io_productive.rs"]
mod session_io_productive;
mod spawn_args;
mod stdio_child;
mod stdio_teardown;
mod stream_log;
mod timing;
mod turn_timeout;

#[cfg(test)]
#[path = "child_stderr_tests.rs"]
mod child_stderr_tests;

#[cfg(test)]
#[path = "drain_idle_tests.rs"]
mod drain_idle_tests;

#[cfg(test)]
#[path = "drain_idle_policy_tests.rs"]
mod drain_idle_policy_tests;

#[cfg(test)]
#[path = "turn_timeout_tests.rs"]
pub(crate) mod turn_timeout_tests;

#[cfg(test)]
pub(crate) use drain_idle::{DrainHealthVerdict, DrainIdleClock};
pub(crate) use drain_idle::{
    DrainIdleHealthCtx, DrainIdleLabels, DrainIdleTurn, await_next_with_idle_in_turn,
};
#[cfg(test)]
pub(crate) use drain_idle::{DrainIdleWaitOpts, await_next_with_idle_using};

pub(crate) use child_process_session::{ChildProcessSession, TurnWait, await_turn_event};
pub(crate) use child_stderr::{start_warning_forward_filtered, take_stdio_forward_stderr};
pub(crate) use json_line_session::JsonLineSession;
#[cfg(test)]
pub(crate) use json_line_session::json_parse_error;
pub(crate) use turn_protocol::{TurnProtocol, consume_turn};
pub(crate) use log_adapter::{feed_do_dm_run_result, handle_stream_event};
pub use session::BridgeSession;
pub use session_io::write_request;
pub(crate) use session_io::{CreateArgs, ResumeArgs, send_create, send_resume, start_mem_watch};
#[cfg(test)]
pub(crate) use session_io_productive::tools_in_flight;
pub use spawn_args::{BridgeSpawnArgs, SDK_BRIDGE_MAX_AGE, ToolCallStart};
pub(crate) use stdio_child::SpawnedStdio;
pub use stdio_child::StdioChild;
pub use stream_log::StreamLog;
pub use timing::{note_sdk_step, record_sdk_usage};
pub(crate) use turn_timeout::TurnTimeoutExtension;

#[cfg(test)]
mod protocol_reexport_tests {
    #[test]
    fn bridge_protocol_is_shared() {
        let _ = crate::backends::cursor_sdk::protocol::encode_request;
        let _ = crate::backends::cursor_sdk::protocol::decode_event;
        let _ = super::write_request;
        let _ = stringify!(BridgeSession);
        let _ = stringify!(BridgeSpawnArgs);
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
