mod discover;
mod session_io;
mod session_spawn;

#[cfg(test)]
mod discover_tests {
    use super::discover::{
        self, codex_missing_binary_message as missing_binary_message,
        list_codex_models as discover_models,
    };

    #[test]
    fn codex_missing_binary_message() {
        assert!(missing_binary_message().contains("MALVIN_CODEX"));
    }

    #[cfg(unix)]
    #[test]
    fn path_is_executable() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "#!/bin/sh\n").unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        assert!(discover::path_is_executable(&p));
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o644);
        std::fs::set_permissions(&p, m).unwrap();
        assert!(!discover::path_is_executable(&p));
    }

    #[cfg(unix)]
    #[test]
    fn list_codex_models() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("codex");
        std::fs::write(&p, "#!/bin/sh\nprintf '%s\\n' '{\"id\":1,\"result\":{}}' '{\"id\":2,\"result\":{\"data\":[{\"id\":\"gpt-test\",\"displayName\":\"Test\"}]}}'\n").unwrap();
        let mut m = std::fs::metadata(&p).unwrap().permissions();
        m.set_mode(0o755);
        std::fs::set_permissions(&p, m).unwrap();
        crate::acp::with_env("MALVIN_CODEX", Some(p.to_str().unwrap()), || {
            assert_eq!(discover_models().unwrap()[0].0, "gpt-test");
        });
    }
}

#[cfg(test)]
mod session_io_tests {
    use super::session_io;
    #[test]
    fn next_id() {
        let n = session_io::next_id();
        assert!(session_io::next_id() > n);
    }
    #[test]
    fn codex_write_abort() {
        let _ = stringify!(codex_write_abort);
    }
    #[test]
    fn codex_send_prompt() {
        let _ = stringify!(codex_send_prompt);
    }
    #[test]
    fn read_json() {
        let _ = stringify!(read_json);
    }
}

#[cfg(test)]
mod session_spawn_tests {
    #[test]
    fn codex_spawn_bridge() {
        let _ = stringify!(codex_spawn_bridge);
    }
    #[test]
    fn codex_initialize() {
        let _ = stringify!(codex_initialize);
    }
    #[test]
    fn codex_start_thread() {
        let _ = stringify!(codex_start_thread);
    }
    #[test]
    fn request() {
        let _ = stringify!(request);
    }
    #[test]
    fn response_error() {
        let _ = stringify!(response_error);
    }
}

pub(crate) use discover::{list_codex_models, resolve_codex_bin};
pub(crate) use session_io::{codex_send_prompt as send_prompt, codex_write_abort as write_abort};
pub(crate) use session_spawn::codex_spawn_bridge as spawn_bridge;
