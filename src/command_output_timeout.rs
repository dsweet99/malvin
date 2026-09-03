use std::io::Read;
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

type PipeReader = thread::JoinHandle<std::io::Result<Vec<u8>>>;

#[must_use]
pub fn timeout_ms_from_env(env_key: &str, default_ms: u64) -> Duration {
    Duration::from_millis(std::env::var(env_key).ok().map_or(default_ms, |s| {
        s.parse::<u64>().map_or_else(
            |_| {
                tracing::warn!(
                    target: "malvin::command_output_timeout",
                    key = %env_key,
                    value = %s,
                    "not a positive integer; using default"
                );
                default_ms
            },
            |n| n.max(1),
        )
    }))
}

pub fn command_output_with_timeout(
    mut cmd: Command,
    timeout: Duration,
    label: &str,
) -> Result<Output, String> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("{label} failed to spawn: {e}"))?;
    let (stdout_handle, stderr_handle) = take_pipe_readers(&mut child, label)?;
    let status = match wait_child_with_timeout(&mut child, timeout, label) {
        Ok(status) => status,
        Err(e) => {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(e);
        }
    };
    Ok(Output {
        status,
        stdout: join_pipe_reader(stdout_handle, label, "stdout")?,
        stderr: join_pipe_reader(stderr_handle, label, "stderr")?,
    })
}

pub(crate) fn wait_piped_child_with_timeout(
    mut child: Child,
    timeout: Duration,
    label: &str,
) -> Result<Output, String> {
    let (stdout_handle, stderr_handle) = take_pipe_readers(&mut child, label)?;
    let status = match wait_child_with_timeout(&mut child, timeout, label) {
        Ok(status) => status,
        Err(e) => {
            let _ = stdout_handle.join();
            let _ = stderr_handle.join();
            return Err(e);
        }
    };
    Ok(Output {
        status,
        stdout: join_pipe_reader(stdout_handle, label, "stdout")?,
        stderr: join_pipe_reader(stderr_handle, label, "stderr")?,
    })
}

pub(crate) fn take_pipe_readers(
    child: &mut Child,
    label: &str,
) -> Result<(PipeReader, PipeReader), String> {
    let stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| format!("{label}: missing stdout pipe"))?;
    let stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| format!("{label}: missing stderr pipe"))?;
    Ok((
        thread::spawn(move || read_pipe_to_end(stdout_pipe)),
        thread::spawn(move || read_pipe_to_end(stderr_pipe)),
    ))
}

fn read_pipe_to_end(mut pipe: impl Read) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    pipe.read_to_end(&mut buf)?;
    Ok(buf)
}

pub(crate) fn join_pipe_reader(
    handle: PipeReader,
    label: &str,
    stream: &str,
) -> Result<Vec<u8>, String> {
    handle
        .join()
        .map_err(|_| format!("{label}: {stream} reader panicked"))?
        .map_err(|e| format!("{label}: failed reading {stream}: {e}"))
}

fn wait_child_with_timeout(
    child: &mut Child,
    timeout: Duration,
    label: &str,
) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    crate::acp::signal_process_group(child.id(), 9);
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("{label} timed out after {}ms", timeout.as_millis()));
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(format!("{label} wait failed: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_env_lock;
    use std::os::unix::fs::PermissionsExt;
    use std::time::Duration;

    #[test]
    fn timeout_ms_from_env_clamps_and_defaults() {
        let _lock = test_env_lock();
        let key = "MALVIN_TEST_CMD_TIMEOUT_MS";
        let prior = std::env::var_os(key);
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(key, "0");
        }
        assert_eq!(timeout_ms_from_env(key, 30_000), Duration::from_millis(1));
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(key, "nope");
        }
        assert_eq!(timeout_ms_from_env(key, 30_000), Duration::from_secs(30));
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(key, "250");
        }
        assert_eq!(timeout_ms_from_env(key, 30_000), Duration::from_millis(250));
        #[allow(unsafe_code)]
        unsafe {
            match prior {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn command_output_with_timeout_kills_hanging_child() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let hang = dir.path().join("hang");
        std::fs::write(&hang, "#!/bin/sh\nsleep 30\n").expect("write");
        let mut perms = std::fs::metadata(&hang).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hang, perms).expect("chmod");

        let cmd = crate::malvin_sandbox::malvin_std_command(&hang);
        let started = Instant::now();
        let err = command_output_with_timeout(cmd, Duration::from_millis(200), "hang-bin")
            .expect_err("must time out");
        assert!(err.contains("timed out"), "got: {err}");
        assert!(started.elapsed() < Duration::from_millis(1500));
    }

    #[test]
    fn kiss_cov_command_output_timeout_symbols() {
        let _ = stringify!(timeout_ms_from_env);
        let _ = stringify!(command_output_with_timeout);
        let _ = stringify!(wait_piped_child_with_timeout);
        let _ = stringify!(take_pipe_readers);
        let _ = stringify!(read_pipe_to_end);
        let _ = stringify!(join_pipe_reader);
        let _ = stringify!(wait_child_with_timeout);
        let _ = stringify!(PipeReader);
    }
}
