use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, Instant};

use super::local_llm_lock::{reclaim_stale_lock, release_lock, try_acquire_lock};
use super::local_llm_ollama::{ensure_provider_running, run_ollama_stop, stop_pid};
use super::local_llm_paths::{idle_duration, manager_lock_path, manager_sock_path};
use super::local_llm_protocol::{ManagerRequest, ManagerResponse};

struct ManagerState {
    last_call: Instant,
    active_model: Option<String>,
    ollama_serve_pid: Option<u32>,
}

pub fn run_local_llm_manager() -> Result<(), String> {
    let sock = manager_sock_path();
    let lock = manager_lock_path();
    if !try_acquire_lock(&lock)? {
        return Ok(());
    }
    let listener = bind_socket(&sock).inspect_err(|_| release_lock(&lock))?;
    let mut state = ManagerState {
        last_call: Instant::now(),
        active_model: None,
        ollama_serve_pid: None,
    };
    let idle = idle_duration();
    let outcome = serve_loop(&listener, &mut state, idle);
    shutdown_provider(&state);
    let _ = fs::remove_file(&sock);
    release_lock(&lock);
    outcome
}

fn bind_socket(sock: &Path) -> Result<UnixListener, String> {
    if let Some(parent) = sock.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
    }
    let _ = fs::remove_file(sock);
    UnixListener::bind(sock).map_err(|e| format!("bind {}: {e}", sock.display()))
}

fn serve_loop(
    listener: &UnixListener,
    state: &mut ManagerState,
    idle: Duration,
) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("nonblocking: {e}"))?;
    loop {
        if state.last_call.elapsed() >= idle {
            return Ok(());
        }
        match listener.accept() {
            Ok((stream, _)) => handle_client(stream, state)?,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("accept: {e}")),
        }
    }
}

fn handle_client(stream: UnixStream, state: &mut ManagerState) -> Result<(), String> {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
        return Ok(());
    }
    let reply = match serde_json::from_str::<ManagerRequest>(line.trim()) {
        Ok(req) => dispatch(req, state),
        Err(e) => ManagerResponse::err(format!("bad request: {e}")),
    };
    write_reply(&stream, &reply)
}

fn dispatch(req: ManagerRequest, state: &mut ManagerState) -> ManagerResponse {
    state.last_call = Instant::now();
    match req {
        ManagerRequest::Ping | ManagerRequest::Hold | ManagerRequest::Release => {
            ManagerResponse::ok_ping(std::process::id())
        }
        ManagerRequest::Ensure { provider, model } => match ensure_provider_running(&provider) {
            Ok(started) => {
                if let Some(pid) = started {
                    state.ollama_serve_pid = Some(pid);
                }
                state.active_model = Some(model);
                ManagerResponse::ok_ensure()
            }
            Err(e) => ManagerResponse::err(e),
        },
    }
}

fn write_reply(mut stream: &UnixStream, reply: &ManagerResponse) -> Result<(), String> {
    let mut line = serde_json::to_string(reply).map_err(|e| e.to_string())?;
    line.push('\n');
    stream.write_all(line.as_bytes()).map_err(|e| e.to_string())
}

fn shutdown_provider(state: &ManagerState) {
    if let Some(model) = state.active_model.as_deref() {
        run_ollama_stop(model);
    }
    if let Some(pid) = state.ollama_serve_pid {
        stop_pid(pid);
    }
}

pub(crate) fn reclaim_stale_manager_files() {
    let sock = manager_sock_path();
    let lock = manager_lock_path();
    reclaim_stale_lock(&lock);
    if super::local_llm_lock::lock_holder_pid(&lock).is_none() {
        let _ = fs::remove_file(&sock);
        let _ = fs::remove_file(&lock);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::with_env;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;

    #[test]
    fn idle_manager_exits_and_clears_sock() {
        let _guard = crate::pi_sdk::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tmp");
        let sock = tmp.path().join("m.sock");
        let lock = tmp.path().join("m.lock");
        with_env(
            "MALVIN_LOCAL_LLM_MANAGER_SOCK",
            Some(sock.to_str().unwrap()),
            || {
                with_env(
                    "MALVIN_LOCAL_LLM_MANAGER_LOCK",
                    Some(lock.to_str().unwrap()),
                    || {
                        with_env("MALVIN_TIME_SINCE_LAST_CALL_SECONDS", Some("1"), || {
                            let handle = std::thread::spawn(run_local_llm_manager);
                            wait_for_sock(&sock);
                            let _ = UnixStream::connect(&sock);
                            handle.join().expect("join").expect("manager ok");
                            assert!(!sock.exists(), "socket removed after idle exit");
                        });
                    },
                );
            },
        );
    }

    #[test]
    fn hold_alone_does_not_block_idle_exit() {
        let _guard = crate::pi_sdk::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tmp");
        let sock = tmp.path().join("h.sock");
        let lock = tmp.path().join("h.lock");
        with_env(
            "MALVIN_LOCAL_LLM_MANAGER_SOCK",
            Some(sock.to_str().unwrap()),
            || {
                with_env(
                    "MALVIN_LOCAL_LLM_MANAGER_LOCK",
                    Some(lock.to_str().unwrap()),
                    || {
                        with_env("MALVIN_TIME_SINCE_LAST_CALL_SECONDS", Some("1"), || {
                            let handle = std::thread::spawn(run_local_llm_manager);
                            wait_for_sock(&sock);
                            send_op(&sock, &ManagerRequest::Hold);
                            handle.join().expect("join").expect("manager ok");
                            assert!(
                                !sock.exists(),
                                "idle must win after last_call even if Hold was never Released"
                            );
                        });
                    },
                );
            },
        );
    }

    #[test]
    fn ping_refreshes_last_call_and_delays_idle() {
        let _guard = crate::pi_sdk::local_llm_test_lock::local_llm_test_env_lock()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tmp");
        let sock = tmp.path().join("p.sock");
        let lock = tmp.path().join("p.lock");
        with_env(
            "MALVIN_LOCAL_LLM_MANAGER_SOCK",
            Some(sock.to_str().unwrap()),
            || {
                with_env(
                    "MALVIN_LOCAL_LLM_MANAGER_LOCK",
                    Some(lock.to_str().unwrap()),
                    || {
                        with_env("MALVIN_TIME_SINCE_LAST_CALL_SECONDS", Some("1"), || {
                            let handle = std::thread::spawn(run_local_llm_manager);
                            wait_for_sock(&sock);
                            std::thread::sleep(Duration::from_millis(600));
                            send_op(&sock, &ManagerRequest::Ping);
                            std::thread::sleep(Duration::from_millis(600));
                            assert!(sock.exists(), "recent ping must delay idle exit");
                            handle.join().expect("join").expect("manager ok");
                            assert!(!sock.exists(), "socket removed after idle");
                        });
                    },
                );
            },
        );
    }

    fn send_op(sock: &Path, req: &ManagerRequest) {
        let mut stream = UnixStream::connect(sock).expect("connect");
        let mut line = serde_json::to_string(req).expect("ser");
        line.push('\n');
        stream.write_all(line.as_bytes()).expect("write");
        let mut buf = [0_u8; 256];
        let _ = stream.read(&mut buf);
    }

    fn wait_for_sock(sock: &PathBuf) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if sock.exists() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("manager sock did not appear");
    }
}
