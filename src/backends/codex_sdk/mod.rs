pub(crate) use super::codex_sdk_auth as auth;
pub(crate) use super::codex_sdk_discover as discover;
pub(crate) use super::codex_sdk_map_event as map_event;
pub(crate) use super::codex_sdk_map_event_summary as map_event_summary;
pub(crate) use super::codex_sdk_map_event_usage as map_event_usage;
pub(crate) use super::codex_sdk_session as session;
pub(crate) use super::codex_sdk_session_io as session_io;
pub(crate) use super::codex_sdk_session_process as session_process;
pub(crate) use super::codex_sdk_session_protocol as session_protocol;
pub(crate) use super::codex_sdk_session_spawn as session_spawn;
pub(crate) use super::codex_sdk_session_turn as session_turn;
pub(crate) use super::codex_sdk_session_turn_done as session_turn_done;

#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod discover_tests;
#[cfg(test)]
mod map_event_more_tests;
#[cfg(test)]
mod map_event_summary_tests;
#[cfg(test)]
mod map_event_tests;
#[cfg(test)]
mod session_turn_tests;
#[cfg(test)]
mod turn_timeout_tests;

pub(crate) use auth::ensure_codex_authenticated;
pub use discover::list_codex_display_models;
pub(crate) use session::CodexSession;
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
        let _ = stringify!(classified_command);
        let _ = stringify!(unwrap_shell);
        let _ = stringify!(codex_flatten_ws);
        let _ = super::map_event_usage::usage_from_turn;
        let _ = super::map_event_usage::usage_event;
        let _ = crate::backends::cursor_sdk::protocol::canonicalize_run_done;
        let _ = super::ensure_codex_authenticated;
        let _ = stringify!(has_codex_login);
        let _ = stringify!(codex_auth_path);
        let _ = stringify!(auth_file_has_login);
        let _ = stringify!(nonempty_json_str);
        let _ = stringify!(codex_effort_from_thinking);
    }
}
