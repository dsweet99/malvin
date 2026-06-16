//! External kiss witnesses for [`super::command`] (must be `*_tests.rs` for kiss).

use std::io::{self, Error, ErrorKind};
use std::path::Path;

use tokio::process::Command;

use super::command::{
    agent_program, apply_acp_tail, apply_api_and_auth, build_agent_acp_command, cmd_env,
    executable_text_busy, forward_parent_env, prepend_standard_path_for_child,
    spawn_agent_acp_child, with_env, AGENT_BIN, BuildAgentAcpCommandArgs,
};

#[test]
fn build_agent_acp_command_uses_bin_override_program() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_eq!(
        cmd.as_std().get_program().to_string_lossy(),
        "/bin/true"
    );
}

#[test]
fn build_agent_acp_command_exercises_all_args_fields() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/false")),
        api_key: Some("test-key"),
        auth_token: Some("test-token"),
        george_acp_lane: Some("lane"),
        model: Some("gpt-4"),
        force: true,
    });
    let args: Vec<String> = cmd
        .as_std()
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args.iter().any(|a| a == "--force"));
    assert!(args.iter().any(|a| a == "--model"));
    assert!(args.iter().any(|a| a == "gpt-4"));
}

#[test]
fn prepend_standard_path_for_child_merges_nonempty_path() {
    with_env("PATH", Some("/custom/bin"), || {
        let mut cmd = Command::new("true");
        prepend_standard_path_for_child(&mut cmd);
        let path = cmd_env(&cmd, "PATH").expect("PATH");
        let merged = path.to_string_lossy();
        assert!(merged.contains("/custom/bin"));
        assert!(merged.contains("/usr/bin"));
    });
}

#[test]
fn prepend_standard_path_for_child_uses_prefix_when_path_missing() {
    with_env("PATH", None, || {
        let mut cmd = Command::new("true");
        prepend_standard_path_for_child(&mut cmd);
        assert_eq!(
            cmd_env(&cmd, "PATH").expect("PATH"),
            "/usr/bin:/bin:/usr/local/bin"
        );
    });
}

#[test]
fn forward_parent_env_skips_empty_values() {
    with_env("HOME", Some(""), || {
        let mut cmd = Command::new("true");
        forward_parent_env(&mut cmd);
        assert!(cmd_env(&cmd, "HOME").is_none());
    });
}

#[test]
fn forward_parent_env_forwards_nonempty_home() {
    with_env("HOME", Some("/tmp/malvin-home"), || {
        let mut cmd = Command::new("true");
        forward_parent_env(&mut cmd);
        assert_eq!(cmd_env(&cmd, "HOME").expect("HOME"), "/tmp/malvin-home");
    });
}

#[test]
fn agent_program_prefers_nonempty_bin_override() {
    assert_eq!(
        agent_program(Some(Path::new("/opt/agent"))),
        "/opt/agent"
    );
    assert_eq!(agent_program(Some(Path::new(""))), AGENT_BIN);
    assert_eq!(agent_program(None), AGENT_BIN);
}

#[test]
fn apply_api_and_auth_sets_key_and_token_when_present() {
    let mut cmd = Command::new("true");
    apply_api_and_auth(&mut cmd, Some("api"), Some("tok"));
    let args: Vec<String> = cmd
        .as_std()
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args.windows(2).any(|w| w == ["--api-key", "api"]));
    assert!(args.windows(2).any(|w| w == ["--auth-token", "tok"]));
    assert_eq!(cmd_env(&cmd, "CURSOR_API_KEY").expect("key"), "api");
    assert_eq!(cmd_env(&cmd, "CURSOR_AUTH_TOKEN").expect("tok"), "tok");
}

#[test]
fn apply_acp_tail_sets_lane_when_nonempty() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut cmd = Command::new("true");
    apply_acp_tail(&mut cmd, tmp.path(), Some("lane-a"));
    assert_eq!(cmd_env(&cmd, "GEORGE_ACP_LANE").expect("lane"), "lane-a");
    assert_eq!(cmd_env(&cmd, "MALVIN_WORKSPACE").expect("work"), tmp.path());
}

#[test]
fn build_agent_acp_command_args_destructure_by_value() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let args = BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/echo")),
        api_key: Some("k"),
        auth_token: Some("t"),
        george_acp_lane: Some("lane"),
        model: Some("m"),
        force: true,
    };
    let touched = std::hint::black_box(args);
    let BuildAgentAcpCommandArgs {
        cwd,
        bin_override,
        api_key,
        auth_token,
        george_acp_lane,
        model,
        force,
    } = touched;
    assert_eq!(cwd, tmp.path());
    assert_eq!(bin_override, Some(Path::new("/bin/echo")));
    assert_eq!(api_key, Some("k"));
    assert_eq!(auth_token, Some("t"));
    assert_eq!(george_acp_lane, Some("lane"));
    assert_eq!(model, Some("m"));
    assert!(force);
}

#[test]
fn kiss_cov_executable_text_busy_kind_and_unix_raw() {
    assert!(executable_text_busy(&Error::new(ErrorKind::ExecutableFileBusy, "b")));
    assert!(!executable_text_busy(&Error::new(ErrorKind::NotFound, "n")));
    #[cfg(unix)]
    assert!(executable_text_busy(&Error::from_raw_os_error(26)));
}

fn write_executable_agent_script(dir: &Path) -> std::path::PathBuf {
    let agent = dir.join("agent");
    std::fs::write(&agent, "#!/bin/sh\nexit 0\n").expect("write agent");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&agent, perms).expect("chmod");
    }
    agent
}

#[test]
fn kiss_cov_spawn_agent_acp_child_success() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            let tmp = tempfile::tempdir().expect("tempdir");
            let agent = write_executable_agent_script(tmp.path());
            let mut cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
                cwd: tmp.path(),
                bin_override: Some(&agent),
                api_key: Some("k"),
                auth_token: None,
                george_acp_lane: None,
                model: None,
                force: false,
            });
            let mut child = spawn_agent_acp_child(&mut cmd).await.expect("spawn");
            assert!(child.id().is_some());
            let _ = child.kill().await;
        });
}

#[test]
fn spawn_agent_acp_child_errors_on_missing_program() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async {
            let mut cmd = Command::new("/no/such/malvin-agent-binary");
            let err = spawn_agent_acp_child(&mut cmd).await.expect_err("spawn");
            assert_ne!(err.kind(), io::ErrorKind::ExecutableFileBusy);
        });
}
