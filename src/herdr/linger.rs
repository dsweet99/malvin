
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::request::{next_request_id, next_seq, release_agent};
use super::send::send_request_reply;

const WORKER_ARG: &str = "__herdr-linger";
const POLL: Duration = Duration::from_millis(500);
const PARENT_EXIT_WAIT: Duration = Duration::from_mins(10);
const MAX_LINGER: Duration = Duration::from_hours(12);
const MAX_CONSECUTIVE_ERRORS: u32 = 20;

#[allow(clippy::missing_const_for_fn)]
fn linger_enabled() -> bool {
    #[cfg(test)]
    {
        false
    }
    #[cfg(not(test))]
    {
        std::env::var_os("MALVIN_NO_HERDR_LINGER").is_none()
    }
}

pub(super) fn spawn_linger(socket_path: &Path, pane_id: &str) -> bool {
    if !linger_enabled() {
        return false;
    }
    let Ok(exe) = crate::self_exe::self_exe() else {
        return false;
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.arg(WORKER_ARG)
        .arg(socket_path)
        .arg(pane_id)
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    super::env::strip_herdr_env(&mut cmd);
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    cmd.spawn().is_ok()
}


#[must_use]
pub fn run_if_worker() -> Option<crate::Exit> {
    let mut args = std::env::args_os().skip(1);
    if args.next()? != WORKER_ARG {
        return None;
    }
    match worker_args(args) {
        (Some(socket), Some(pane), Some(pid)) => {
            watch(&socket, &pane, pid);
            Some(crate::Exit::Success)
        }
        (Some(socket), Some(pane), None) => {
            release(&socket, &pane);
            Some(crate::Exit::Failure)
        }
        _ => Some(crate::Exit::Failure),
    }
}

fn worker_args(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> (Option<PathBuf>, Option<String>, Option<u32>) {
    let socket = args.next().map(PathBuf::from);
    let pane = args.next().and_then(|a| a.into_string().ok());
    let pid = args
        .next()
        .and_then(|a| a.into_string().ok())
        .and_then(|a| a.parse::<u32>().ok());
    (socket, pane, pid)
}

fn watch(socket: &Path, pane: &str, parent_pid: u32) {
    wait_for_parent_exit(parent_pid);
    std::thread::sleep(Duration::from_secs(1));
    if linger_ending(socket, pane, POLL, MAX_LINGER) == Verdict::Release {
        release(socket, pane);
    }
}

fn release(socket: &Path, pane: &str) {
    let _ = send_request_reply(socket, &release_agent(pane, next_seq()));
}

fn linger_ending(socket: &Path, pane: &str, poll: Duration, max_linger: Duration) -> Verdict {
    let Some(baseline) = read_visible(socket, pane) else {
        return Verdict::Release;
    };
    let deadline = Instant::now() + max_linger;
    let mut errors = 0_u32;
    while Instant::now() < deadline {
        std::thread::sleep(poll);
        match poll_pane(socket, pane, &baseline) {
            Ok(Verdict::Wait) => errors = 0,
            Ok(ending) => return ending,
            Err(()) => {
                errors += 1;
                if errors >= MAX_CONSECUTIVE_ERRORS {
                    return Verdict::Release;
                }
            }
        }
    }
    Verdict::Release
}

fn wait_for_parent_exit(pid: u32) {
    let deadline = Instant::now() + PARENT_EXIT_WAIT;
    while crate::agent_process::pid_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(POLL);
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Wait,
    Release,
    Leave,
}

fn poll_pane(socket: &Path, pane: &str, baseline: &str) -> Result<Verdict, ()> {
    let reply = match send_request_reply(socket, &pane_request("pane.get", pane, None)) {
        Ok(v) => v,
        Err(e) if e.starts_with("pane_not_found") => return Ok(Verdict::Leave),
        Err(_) => return Err(()),
    };
    let info = &reply["pane"];
    if info["agent_status"] == "working" {
        return Ok(Verdict::Leave);
    }
    if info["focused"] != true {
        return Ok(Verdict::Wait);
    }
    let text = read_visible(socket, pane).ok_or(())?;
    Ok(if text == baseline {
        Verdict::Wait
    } else {
        Verdict::Release
    })
}

fn read_visible(socket: &Path, pane: &str) -> Option<String> {
    let reply =
        send_request_reply(socket, &pane_request("pane.read", pane, Some("visible"))).ok()?;
    reply["read"]["text"].as_str().map(str::to_string)
}

fn pane_request(method: &str, pane: &str, source: Option<&str>) -> Value {
    let mut params = json!({ "pane_id": pane });
    if let Some(source) = source {
        params["source"] = json!(source);
    }
    json!({ "id": next_request_id(), "method": method, "params": params })
}

#[cfg(test)]
mod tests {
    use super::{Verdict, poll_pane};
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    fn serve(sock: &std::path::Path, focused: bool, status: &'static str, text: &'static str) {
        serve_n(sock, 4, focused, status, text);
    }

    fn serve_n(
        sock: &std::path::Path,
        n: usize,
        focused: bool,
        status: &'static str,
        text: &'static str,
    ) {
        let listener = UnixListener::bind(sock).unwrap();
        std::thread::spawn(move || {
            for conn in listener.incoming().take(n) {
                let mut conn = conn.unwrap();
                let mut line = String::new();
                BufReader::new(&conn).read_line(&mut line).unwrap();
                let reply = if line.contains("pane.get") {
                    format!(
                        r#"{{"id":"1","result":{{"pane":{{"focused":{focused},"agent_status":"{status}"}}}}}}"#
                    )
                } else {
                    format!(r#"{{"id":"1","result":{{"read":{{"text":"{text}"}}}}}}"#)
                };
                conn.write_all(format!("{reply}\n").as_bytes()).unwrap();
            }
        });
    }

    fn verdict(focused: bool, status: &'static str, text: &'static str) -> &'static str {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("h.sock");
        serve(&sock, focused, status, text);
        match poll_pane(&sock, "p", "base").ok().unwrap() {
            Verdict::Wait => "wait",
            Verdict::Release => "release",
            Verdict::Leave => "leave",
        }
    }

    fn ending(sock: &std::path::Path, max_linger_ms: u64) -> &'static str {
        let poll = std::time::Duration::from_millis(1);
        let max = std::time::Duration::from_millis(max_linger_ms);
        match super::linger_ending(sock, "p", poll, max) {
            Verdict::Wait => "wait",
            Verdict::Release => "release",
            Verdict::Leave => "leave",
        }
    }

    fn worker_releases_on_failed_baseline_error_limit_and_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let absent = dir.path().join("absent.sock");
        assert_eq!(ending(&absent, 1000), "release");
        let sock = dir.path().join("h.sock");
        serve_n(&sock, 1, false, "idle", "base");
        assert_eq!(ending(&sock, 4000), "release");
        let sock = dir.path().join("w.sock");
        serve_n(&sock, 10_000, false, "idle", "base");
        assert_eq!(ending(&sock, 50), "release");
        let sock = dir.path().join("l.sock");
        serve_n(&sock, 2, false, "working", "base");
        assert_eq!(ending(&sock, 1000), "leave");
    }

    #[test]
    fn releases_only_when_focused_and_screen_changed_and_not_taken_over() {
        assert_eq!(verdict(false, "idle", "new"), "wait");
        assert_eq!(verdict(true, "idle", "base"), "wait");
        assert_eq!(verdict(true, "idle", "new"), "release");
        assert_eq!(verdict(true, "working", "new"), "leave");
        worker_releases_on_failed_baseline_error_limit_and_timeout();
    }
}
