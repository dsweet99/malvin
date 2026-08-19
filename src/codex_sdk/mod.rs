mod discover;
mod session_io;
mod session_spawn;

#[cfg(test)]
mod discover_tests;

pub(crate) use discover::{list_codex_models, resolve_codex_bin};
pub(crate) use session_io::{codex_send_prompt as send_prompt, codex_write_abort as write_abort};
pub(crate) use session_spawn::codex_spawn_bridge as spawn_bridge;

#[cfg(test)]
mod discover_tests_inline {
    use super::discover::path_is_executable;

    #[test]
    fn codex_path_is_executable_witness() {
        let _ = path_is_executable;
    }
}
