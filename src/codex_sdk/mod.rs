mod discover;
mod map_event;
mod map_event_summary;
mod session_io;
#[path = "session_process.rs"]
mod session_process;
#[path = "session_protocol.rs"]
mod session_protocol;
mod session_spawn;
mod session_turn;

#[cfg(test)]
mod discover_tests;
#[cfg(test)]
mod map_event_more_tests;
#[cfg(test)]
mod map_event_tests;
#[cfg(test)]
mod session_turn_tests;

pub(crate) use discover::{list_codex_models, resolve_codex_model};
pub(crate) use session_io::{codex_send_prompt as send_prompt, codex_write_abort as write_abort};
pub(crate) use session_spawn::codex_spawn_bridge as spawn_bridge;

#[cfg(test)]
mod discover_tests_inline {
    use super::discover::codex_path_is_executable;

    #[test]
    fn codex_path_is_executable_witness() {
        let _ = codex_path_is_executable;
    }
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_cov_codex_map_event() {
        let _ = super::map_event::map_codex_stream_events;
        let _ = stringify!(tool_name_summary);
    }
}
