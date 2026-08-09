//! Shared Cursor/Prime JSONL bridge core (session, IO, log adapters, timing).

mod log_adapter;
mod log_adapter_tool;
mod session;
mod session_io;
mod timing;

pub use session::{BridgeSession, BridgeSpawnArgs, ToolCallStart, SDK_BRIDGE_MAX_AGE};
pub use session_io::write_request;
pub(crate) use session_io::{
    send_create, send_resume, start_mem_watch, CreateArgs, ResumeArgs,
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
        let _ = stringify!(ToolCallStart);
        let _ = stringify!(SDK_BRIDGE_MAX_AGE);
        let _ = stringify!(send_create);
        let _ = stringify!(send_resume);
        let _ = stringify!(start_mem_watch);
        let _ = stringify!(note_sdk_step);
        let _ = stringify!(record_sdk_usage);
    }
}
