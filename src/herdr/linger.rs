
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
    let Some(exe) = worker_exe() else {
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

fn worker_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let stripped = exe.to_str()?.strip_suffix(" (deleted)").map(PathBuf::from);
    Some(stripped.unwrap_or(exe))
}

#[must_use]
pub fn run_if_worker() -> Option<crate::Exit> {
    let mut args = std::env::args_os().skip(1);
    if args.next()? != WORKER_ARG {
        return None;
    }
    let socket = args.next().map(PathBuf::from);
    let pane = args.next().and_then(|a| a.into_string().ok());
    let pid = args
        .next()
        .and_then(|a| a.into_string().ok())
        .and_then(|a| a.parse::<u32>().ok());
    if let (Some(socket), Some(pane), Some(pid)) = (socket, pane, pid) {
        watch(&socket, &pane, pid);
        Some(crate::Exit::Success)
    } else {
        Some(crate::Exit::Failure)
    }
}

fn watch(socket: &Path, pane: &str, parent_pid: u32) {
    wait_for_parent_exit(parent_pid);
    std::thread::sleep(Duration::from_secs(1));
    let Some(baseline) = read_visible(socket, pane) else {
        return;
    };
    let deadline = Instant::now() + MAX_LINGER;
    let mut errors = 0_u32;
    while Instant::now() < deadline {
        std::thread::sleep(POLL);
        match poll_pane(socket, pane, &baseline) {
            Ok(Verdict::Wait) => errors = 0,
            Ok(Verdict::Release) => {
                let _ = send_request_reply(socket, &release_agent(pane, next_seq()));
                return;
            }
            Ok(Verdict::Leave) => return,
            Err(()) => {
                errors += 1;
                if errors >= MAX_CONSECUTIVE_ERRORS {
                    return;
                }
            }
        }
    }
}

fn wait_for_parent_exit(pid: u32) {
    let proc_dir = PathBuf::from(format!("/proc/{pid}"));
    let deadline = Instant::now() + PARENT_EXIT_WAIT;
    while proc_dir.exists() && Instant::now() < deadline {
        std::thread::sleep(POLL);
    }
}

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
        let listener = UnixListener::bind(sock).unwrap();
        std::thread::spawn(move || {
            for conn in listener.incoming().take(4) {
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

    #[test]
    fn releases_only_when_focused_and_screen_changed_and_not_taken_over() {
        assert_eq!(verdict(false, "idle", "new"), "wait");
        assert_eq!(verdict(true, "idle", "base"), "wait");
        assert_eq!(verdict(true, "idle", "new"), "release");
        assert_eq!(verdict(true, "working", "new"), "leave");
    }
}
