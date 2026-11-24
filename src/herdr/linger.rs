use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::request::{next_seq, pane_get, pane_read, release_agent};
use super::send::send_request_reply;
use crate::self_worker::Worker;

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
        !crate::env_flag::env_flag("MALVIN_NO_HERDR_LINGER")
    }
}

pub(super) fn spawn_linger(socket_path: &Path, pane_id: &str) -> bool {
    if !linger_enabled() {
        return false;
    }
    let pid = std::process::id().to_string();
    let args = [
        socket_path.as_os_str(),
        OsStr::new(pane_id),
        OsStr::new(&pid),
    ];
    crate::self_worker::spawn_detached(Worker::HerdrLinger, args).is_ok()
}

pub fn run_worker(args: Vec<OsString>) -> Result<(), String> {
    match worker_args(args) {
        (Some(socket), Some(pane), Some(pid)) => {
            watch(&socket, &pane, pid);
            Ok(())
        }
        (Some(socket), Some(pane), None) => {
            release(&socket, &pane);
            Err("missing parent pid; released the pane".to_string())
        }
        _ => Err(crate::self_worker::usage(
            Worker::HerdrLinger,
            "SOCKET PANE PARENT_PID",
        )),
    }
}

fn worker_args(args: Vec<OsString>) -> (Option<PathBuf>, Option<String>, Option<u32>) {
    let mut args = args.into_iter();
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
    let reply = match send_request_reply(socket, &pane_get(pane)) {
        Ok(v) => v,
        Err(e) if e.has_code("pane_not_found") => return Ok(Verdict::Leave),
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
    let reply = send_request_reply(socket, &pane_read(pane, "visible")).ok()?;
    reply["read"]["text"].as_str().map(str::to_string)
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
