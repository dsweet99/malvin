#[cfg(all(unix, target_os = "linux"))]
use std::io::Read;
#[cfg(all(unix, target_os = "linux"))]
use std::path::{Path, PathBuf};
#[cfg(all(unix, target_os = "linux"))]
use std::time::Instant;

#[cfg(all(unix, target_os = "linux"))]
pub struct PtyEnv {
    pub root: tempfile::TempDir,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub bin_dir: PathBuf,
    pub mock: PathBuf,
}

#[cfg(all(unix, target_os = "linux"))]
fn split_malvin_args_line(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quote = !in_quote,
            ' ' | '\t' if !in_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

#[cfg(all(unix, target_os = "linux"))]
fn malvin_pty_command(
    env: &PtyEnv,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> portable_pty::CommandBuilder {
    use portable_pty::CommandBuilder;

    use super::integration_cli_args::{FAST_GATE_LOOP_TEST_ARGS, INTEGRATION_TEST_MALVIN_ARGS};

    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.cwd(&env.workspace);
    cmd.env(
        "PATH",
        format!(
            "{}:{}",
            env.bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        ),
    );
    cmd.env("HOME", env.home.as_os_str());
    cmd.env("CURSOR_AGENT_API_KEY", "test");
    cmd.env("MALVIN_AGENT_ACP_BIN", env.mock.as_os_str());
    cmd.env("MALVIN_TEST_NO_REAL_AGENT", "1");
    cmd.env("MALLOC_ARENA_MAX", "2");
    cmd.env_remove("NO_COLOR");
    if let Some(cols) = columns {
        cmd.env("COLUMNS", cols);
    }
    for arg in INTEGRATION_TEST_MALVIN_ARGS {
        cmd.arg(arg);
    }
    for arg in FAST_GATE_LOOP_TEST_ARGS {
        cmd.arg(arg);
    }
    for arg in split_malvin_args_line(malvin_args_line) {
        cmd.arg(arg);
    }
    cmd
}

#[cfg(all(unix, target_os = "linux"))]
fn wait_pty_child(
    child: &mut Box<dyn portable_pty::Child + Send + Sync>,
    deadline: Instant,
) -> portable_pty::ExitStatus {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    panic!(
                        "pty malvin timed out after {:?}",
                        super::MALVIN_TEST_CMD_TIMEOUT
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(err) => panic!("wait pty child: {err}"),
        }
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn std_exit_status(status: &portable_pty::ExitStatus) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(status.exit_code().cast_signed())
}

#[cfg(all(unix, target_os = "linux"))]
fn read_pty_master(mut reader: Box<dyn Read + Send>) -> Vec<u8> {
    let mut stdout = Vec::new();
    reader
        .read_to_end(&mut stdout)
        .expect("read pty master");
    stdout
}

#[cfg(all(unix, target_os = "linux"))]
fn open_test_pty(cols: u16) -> portable_pty::PtyPair {
    use portable_pty::{native_pty_system, PtySize, PtySystem};
    native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("openpty")
}

#[cfg(all(unix, target_os = "linux"))]
fn spawn_malvin_in_pty(
    pair: &portable_pty::PtyPair,
    env: &PtyEnv,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> Box<dyn portable_pty::Child + Send + Sync> {
    let cmd = malvin_pty_command(env, malvin_args_line, columns);
    pair.slave.spawn_command(cmd).expect("spawn malvin under pty")
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_malvin_under_openpty(
    env: &PtyEnv,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> std::process::Output {
    let cols = columns.and_then(|value| value.parse().ok()).unwrap_or(80);
    let pair = open_test_pty(cols);
    let mut child = spawn_malvin_in_pty(&pair, env, malvin_args_line, columns);
    drop(pair.slave);
    drop(pair.master.take_writer());

    let reader = pair.master.try_clone_reader().expect("pty reader");
    let read_handle = std::thread::spawn(move || read_pty_master(reader));

    let deadline = Instant::now() + super::MALVIN_TEST_CMD_TIMEOUT;
    let status = std_exit_status(&wait_pty_child(&mut child, deadline));
    let stdout = read_handle.join().expect("join pty reader");
    drop(pair.master);

    std::process::Output {
        status,
        stdout,
        stderr: Vec::new(),
    }
}
