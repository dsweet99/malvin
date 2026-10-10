pub(crate) use super::bridge_sdk_child_process_session as child_process_session;
pub(crate) use super::bridge_sdk_child_stderr as child_stderr;
pub(crate) use super::bridge_sdk_drain_idle as drain_idle;
pub(crate) use super::bridge_sdk_json_line_session as json_line_session;
pub(crate) use super::bridge_sdk_log_adapter as log_adapter;
pub(crate) use super::bridge_sdk_log_adapter_tool as log_adapter_tool;
pub(crate) use super::bridge_sdk_session as session;
pub(crate) use super::bridge_sdk_session_handshake as session_handshake;
pub(crate) use super::bridge_sdk_session_io as session_io;
pub(crate) use super::bridge_sdk_session_io_productive as session_io_productive;
pub(crate) use super::bridge_sdk_spawn_args as spawn_args;
pub(crate) use super::bridge_sdk_stdio_child as stdio_child;
pub(crate) use super::bridge_sdk_stdio_teardown as stdio_teardown;
pub(crate) use super::bridge_sdk_stdout_coalesce_buf as stdout_coalesce_buf;
pub(crate) use super::bridge_sdk_stream_log as stream_log;
pub(crate) use super::bridge_sdk_timing as timing;
pub(crate) use super::bridge_sdk_turn_protocol as turn_protocol;
pub(crate) use super::bridge_sdk_turn_timeout as turn_timeout;

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
pub(crate) use turn_protocol::{TurnProtocol, consume_turn};

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
