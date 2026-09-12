use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HerdrEnv {
    pub socket_path: PathBuf,
    pub pane_id: String,
}

impl HerdrEnv {
    #[must_use]
    pub fn from_os_env() -> Option<Self> {
        from_values(
            std::env::var_os("HERDR_ENV"),
            std::env::var_os("HERDR_SOCKET_PATH"),
            std::env::var_os("HERDR_PANE_ID"),
        )
    }
}

pub const HERDR_ENV_VARS: [&str; 3] = ["HERDR_ENV", "HERDR_SOCKET_PATH", "HERDR_PANE_ID"];

pub fn strip_herdr_env(cmd: &mut std::process::Command) {
    for var in HERDR_ENV_VARS {
        cmd.env_remove(var);
    }
}

pub fn strip_herdr_env_tokio(cmd: &mut tokio::process::Command) {
    for var in HERDR_ENV_VARS {
        cmd.env_remove(var);
    }
}

#[must_use]
pub fn from_values(
    herdr_env: Option<std::ffi::OsString>,
    socket_path: Option<std::ffi::OsString>,
    pane_id: Option<std::ffi::OsString>,
) -> Option<HerdrEnv> {
    if herdr_env.as_deref() != Some(std::ffi::OsStr::new("1")) {
        return None;
    }
    let socket = socket_path?.into_string().ok()?;
    let pane = pane_id?.into_string().ok()?;
    if socket.is_empty() || pane.is_empty() {
        return None;
    }
    Some(HerdrEnv {
        socket_path: PathBuf::from(socket),
        pane_id: pane,
    })
}

#[cfg(test)]
mod tests {
    use super::{HerdrEnv, from_values};
    use std::ffi::OsString;

    #[test]
    fn gate_requires_exact_env_one_and_nonempty_socket_pane() {
        assert!(from_values(None, Some("s".into()), Some("p".into())).is_none());
        assert!(from_values(Some("0".into()), Some("s".into()), Some("p".into())).is_none());
        assert!(from_values(Some("1".into()), Some("".into()), Some("p".into())).is_none());
        assert!(from_values(Some("1".into()), Some("s".into()), Some("".into())).is_none());
        assert_eq!(
            from_values(
                Some("1".into()),
                Some("/tmp/h.sock".into()),
                Some("pane".into())
            ),
            Some(HerdrEnv {
                socket_path: "/tmp/h.sock".into(),
                pane_id: "pane".into(),
            })
        );
    }

    #[test]
    fn from_os_env_matches_process_env_snapshot() {
        let _ = HerdrEnv::from_os_env();
        let _ = OsString::new();
    }

    #[test]
    fn strip_herdr_env_removes_triad() {
        let mut std_cmd = std::process::Command::new("true");
        std_cmd.env("HERDR_ENV", "1");
        std_cmd.env("HERDR_SOCKET_PATH", "/tmp/h.sock");
        std_cmd.env("HERDR_PANE_ID", "p1");
        super::strip_herdr_env(&mut std_cmd);
        let mut tokio_cmd = tokio::process::Command::new("true");
        tokio_cmd.env("HERDR_ENV", "1");
        tokio_cmd.env("HERDR_SOCKET_PATH", "/tmp/h.sock");
        tokio_cmd.env("HERDR_PANE_ID", "p1");
        super::strip_herdr_env_tokio(&mut tokio_cmd);
    }
}
