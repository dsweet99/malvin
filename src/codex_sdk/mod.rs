mod discover;
mod session_io;
#[path = "session_process.rs"]
mod session_process;
#[path = "session_protocol.rs"]
mod session_protocol;
mod session_spawn;

#[cfg(test)]
mod discover_tests;

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
