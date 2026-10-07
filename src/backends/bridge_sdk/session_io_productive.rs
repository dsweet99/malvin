use super::bridge_sdk::stream_log::StreamLog;

pub(crate) fn tools_in_flight(session: &StreamLog) -> bool {
    !session
        .tool_starts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_empty()
}
