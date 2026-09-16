#![allow(unsafe_code)]

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::local_llm_ollama::SERVE_WAIT;
use super::local_llm_paths::{INTERNAL_MANAGER_FLAG, manager_sock_path};
use super::local_llm_protocol::{ManagerRequest, ManagerResponse};

const CONNECT_WAIT: Duration = Duration::from_secs(10);
const CONNECT_POLL: Duration = Duration::from_millis(50);
const CONNECT_IO: Duration = Duration::from_secs(2);
const QUICK_IO: Duration = Duration::from_secs(2);

fn ensure_io_timeout() -> Duration {
    SERVE_WAIT + Duration::from_secs(5)
}

pub(crate) fn ensure_via_manager(provider: &str, model: &str) -> Result<(), String> {
    match try_ensure(provider, model) {
        Ok(()) => Ok(()),
        Err(e) if is_unreachable(&e) => {
            spawn_manager()?;
            wait_for_socket(&manager_sock_path())?;
            try_ensure(provider, model)
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn hold_via_manager() -> Result<(), String> {
    quick_ok(ManagerRequest::Hold)?;
    super::local_llm_keepalive::start_keepalive();
    Ok(())
}

pub(crate) fn release_via_manager() -> Result<(), String> {
    super::local_llm_keepalive::stop_keepalive();
    quick_ok(ManagerRequest::Release)
}

pub(crate) fn touch_via_manager() -> Result<(), String> {
    quick_ok(ManagerRequest::Ping)
}

fn quick_ok(req: ManagerRequest) -> Result<(), String> {
    let resp = round_trip(&manager_sock_path(), &req, QUICK_IO)?;
    if resp.ok {
        Ok(())
    } else {
        Err(resp.error.unwrap_or_else(|| "manager request failed".into()))
    }
}

fn try_ensure(provider: &str, model: &str) -> Result<(), String> {
    let req = ManagerRequest::Ensure {
        provider: provider.to_string(),
        model: model.to_string(),
    };
    let resp = round_trip(&manager_sock_path(), &req, ensure_io_timeout())?;
    if resp.ok {
        Ok(())
    } else {
        Err(resp.error.unwrap_or_else(|| "manager ensure failed".into()))
    }
}

fn is_unreachable(err: &str) -> bool {
    err.contains("connect ") || err.contains("socket not ready")
}

fn round_trip(sock: &Path, req: &ManagerRequest, io: Duration) -> Result<ManagerResponse, String> {
    let mut stream = connect(sock, io)?;
    let mut line = serde_json::to_string(req).map_err(|e| e.to_string())?;
    line.push('\n');
    stream
        .write_all(line.as_bytes())
        .map_err(|e| format!("write manager: {e}"))?;
    let mut buf = [0_u8; 4096];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("read manager: {e}"))?;
    let text = std::str::from_utf8(&buf[..n]).map_err(|e| e.to_string())?;
    let first = text.lines().next().unwrap_or("").trim();
    serde_json::from_str(first).map_err(|e| format!("manager reply: {e}"))
}

fn connect(sock: &Path, io: Duration) -> Result<UnixStream, String> {
    let stream =
        UnixStream::connect(sock).map_err(|e| format!("connect {}: {e}", sock.display()))?;
    stream.set_read_timeout(Some(io)).map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(io))
        .map_err(|e| e.to_string())?;
    Ok(stream)
}

fn spawn_manager() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let mut cmd = Command::new(exe);
    cmd.arg(INTERNAL_MANAGER_FLAG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach_session(&mut cmd);
    forward_manager_env(&mut cmd);
    let child = cmd
        .spawn()
        .map_err(|e| format!("spawn local llm manager: {e}"))?;
    std::mem::forget(child);
    Ok(())
}

fn forward_manager_env(cmd: &mut Command) {
    for key in [
        "MALVIN_LOCAL_LLM_MANAGER_SOCK",
        "MALVIN_LOCAL_LLM_MANAGER_LOCK",
        "MALVIN_TIME_SINCE_LAST_CALL_SECONDS",
        "MALVIN_OLLAMA",
        "MALVIN_HOME",
        "HOME",
    ] {
        if let Ok(v) = std::env::var(key) {
            cmd.env(key, v);
        }
    }
}

#[cfg(unix)]
fn detach_session(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            match libc::fork() {
                -1 => Err(std::io::Error::last_os_error()),
                0 => Ok(()),
                _ => {
                    libc::_exit(0);
                }
            }
        });
    }
}

#[cfg(not(unix))]
fn detach_session(_cmd: &mut Command) {}

fn wait_for_socket(sock: &Path) -> Result<(), String> {
    let deadline = Instant::now() + CONNECT_WAIT;
    while Instant::now() < deadline {
        if sock.exists()
            && UnixStream::connect(sock)
                .ok()
                .and_then(|s| {
                    s.set_read_timeout(Some(CONNECT_IO)).ok()?;
                    Some(s)
                })
                .is_some()
        {
            return Ok(());
        }
        std::thread::sleep(CONNECT_POLL);
    }
    Err(format!(
        "local llm manager socket not ready: {}",
        sock.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::super::local_llm_daemon::run_local_llm_manager;
    use super::*;
    use crate::acp::with_env;

    #[test]
    fn ensure_reaches_running_manager() {
        let _guard = crate::pi_sdk::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tmp");
        let sock = tmp.path().join("c.sock");
        let lock = tmp.path().join("c.lock");
        let sock_s = sock.to_str().unwrap().to_string();
        let lock_s = lock.to_str().unwrap().to_string();
        with_env("MALVIN_LOCAL_LLM_MANAGER_SOCK", Some(&sock_s), || {
            with_env("MALVIN_LOCAL_LLM_MANAGER_LOCK", Some(&lock_s), || {
                with_env("MALVIN_TIME_SINCE_LAST_CALL_SECONDS", Some("1"), || {
                    let handle = std::thread::spawn(run_local_llm_manager);
                    wait_for_socket(&sock).expect("sock");
                    let err = try_ensure("llamacpp", "x").expect_err("down");
                    assert!(err.contains("auto-start only"), "{err}");
                    assert!(
                        ensure_via_manager("llamacpp", "x").is_err(),
                        "app error must not spawn forever"
                    );
                    handle.join().expect("join").expect("manager ok");
                });
            });
        });
    }
}
