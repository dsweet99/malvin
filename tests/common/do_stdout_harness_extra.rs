#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use super::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_do_streaming_update_js, command_output_with_timeout,
    test_home_workspace, write_mock_executable,
};

#[cfg(unix)]
pub fn prepare_do_workspace_src_remove_kissconfig()
-> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let (root, home, workspace) = test_home_workspace();
    std::fs::create_dir_all(workspace.join("src")).expect("mkdir src");
    std::fs::write(workspace.join("src/main.rs"), "fn main() {}").expect("write source");
    let _ = std::fs::remove_file(workspace.join(".kissconfig"));
    (root, home, workspace)
}

#[cfg(unix)]
pub fn chmod755(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub fn write_fake_kiss_clamp_installs_kissconfig(kiss: &Path, kissconfig: &Path, marker: &Path) {
    std::fs::write(
        kiss,
        format!(
            "#!/usr/bin/env sh\nprintf 'k\\n' > '{}'\nprintf 'called' > '{}'\n",
            kissconfig.display(),
            marker.display()
        ),
    )
    .expect("write fake kiss");
    chmod755(kiss);
}

#[cfg(unix)]
pub fn write_fake_kiss_marker_fail(kiss: &Path, marker: &Path) {
    std::fs::write(
        kiss,
        format!(
            "#!/usr/bin/env sh\nprintf 'bad' > '{}'\nexit 1\n",
            marker.display()
        ),
    )
    .expect("write fake kiss");
    chmod755(kiss);
}

#[cfg(unix)]
pub struct DoBinCtx {
    pub root: tempfile::TempDir,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub bin_dir: PathBuf,
    pub mock: PathBuf,
}

#[cfg(unix)]
pub fn prepare_do_bin_streaming_mock(mock_js: &str) -> DoBinCtx {
    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock, mock_js);
    DoBinCtx {
        root,
        home,
        workspace,
        bin_dir,
        mock,
    }
}

#[cfg(unix)]
pub fn run_do_say_hi_path_prefixed(ctx: &DoBinCtx) -> std::process::Output {
    let path = format!(
        "{}:{}",
        ctx.bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(&ctx.workspace)
            .env("HOME", &ctx.home)
            .env("PATH", path)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", &ctx.mock)
            .args(["do", "say hi"]),
        MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin do")
}

#[cfg(unix)]
pub fn prepare_do_auto_clamp_case(
    mock_js: &str,
) -> (DoBinCtx, std::path::PathBuf, std::path::PathBuf) {
    let ctx = prepare_do_bin_streaming_mock(mock_js);
    std::fs::create_dir_all(ctx.workspace.join("src")).expect("mkdir src");
    std::fs::write(ctx.workspace.join("src/main.rs"), "fn main() {}").expect("write source");
    let _ = std::fs::remove_file(ctx.workspace.join(".kissconfig"));
    let marker = ctx.workspace.join("kiss_clamp_called.txt");
    let kissconfig = ctx.workspace.join(".kissconfig");
    let kiss = ctx.bin_dir.join("kiss");
    write_fake_kiss_clamp_installs_kissconfig(&kiss, &kissconfig, &marker);
    (ctx, marker, kissconfig)
}

#[cfg(unix)]
pub fn prepare_do_skip_clamp_case(
    mock_js: &str,
    existing_kiss: &str,
) -> (DoBinCtx, std::path::PathBuf) {
    let ctx = prepare_do_bin_streaming_mock(mock_js);
    std::fs::create_dir_all(ctx.workspace.join("src")).expect("mkdir src");
    std::fs::write(ctx.workspace.join("src/main.rs"), "fn main() {}").expect("write source");
    std::fs::write(ctx.workspace.join(".kissconfig"), existing_kiss).expect("write kissconfig");
    let marker = ctx.workspace.join("kiss_clamp_called.txt");
    let kiss = ctx.bin_dir.join("kiss");
    write_fake_kiss_marker_fail(&kiss, &marker);
    (ctx, marker)
}

#[cfg(unix)]
pub fn run_malvin_do_home_workspace(
    workspace: &Path,
    home: &Path,
    mock: &Path,
) -> std::process::Output {
    command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(workspace)
            .env("HOME", home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", mock)
            .args(["do", "say hi"]),
        MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin do")
}
