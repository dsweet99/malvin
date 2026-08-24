use super::{SOCKET_TIMEOUT, classify_reply, send_request, send_request_checked};
use serde_json::json;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::Instant;

fn bind_or_skip(path: &std::path::Path) -> Option<UnixListener> {
    match UnixListener::bind(path) {
        Ok(listener) => Some(listener),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping Unix-socket send test: bind denied: {error}");
            None
        }
        Err(error) => panic!("bind: {error}"),
    }
}

#[test]
fn send_request_writes_ndjson_line_to_unix_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("t.sock");
    let Some(listener) = bind_or_skip(&sock) else {
        return;
    };
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut conn, _) = listener.accept().expect("accept");
        let mut buf = Vec::new();
        let _ = conn.read_to_end(&mut buf);
        let _ = conn.write_all(br#"{"result":{"type":"ok"}}"#);
        let _ = tx.send(buf);
    });
    let req = json!({"id":"t","method":"ping","params":{}});
    send_request(&sock, &req);
    let got = rx.recv_timeout(Duration::from_secs(2)).expect("recv");
    let text = String::from_utf8_lossy(&got);
    assert!(text.ends_with('\n'), "expected NDJSON newline: {text:?}");
    assert!(text.contains("\"method\":\"ping\""));
    let _ = SOCKET_TIMEOUT;
    let _ = send_request_checked;
}

#[test]
fn send_request_swallows_missing_socket() {
    send_request(std::path::Path::new("/no/such/herdr.sock"), &json!({}));
    assert!(send_request_checked(std::path::Path::new("/no/such/herdr.sock"), &json!({})).is_err());
}

#[test]
fn classify_reply_detects_herdr_error_json() {
    assert!(classify_reply(br#"{"id":"1","error":{"code":"x","message":"no"}}"#).is_err());
    assert!(classify_reply(br#"{"result":{"type":"ok"}}"#).is_ok());
    assert!(classify_reply(b"").is_err());
    assert!(classify_reply(b"HTTP/1.1 500").is_err());
    assert!(classify_reply(br#"{"id":"1"}"#).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn send_request_returns_when_accept_queue_is_wedged() {
    let Some(wedge) = WedgedUnixSocket::new() else {
        return;
    };
    let req = json!({"id": "t", "method": "ping", "params": {}});
    assert_connect_times_out(|| send_request_checked(&wedge.path, &req));
    assert_returns_within_budget(|| send_request(&wedge.path, &req));
}

#[cfg(target_os = "linux")]
struct WedgedUnixSocket {
    path: std::path::PathBuf,
    _dir: tempfile::TempDir,
    _listener: UnixListener,
    _holders: [UnixStream; 2],
}

#[cfg(target_os = "linux")]
impl WedgedUnixSocket {
    fn new() -> Option<Self> {
        #![allow(unsafe_code)]
        use std::os::fd::AsRawFd;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("wedge.sock");
        let listener = bind_or_skip(&path)?;
        assert_eq!(unsafe { libc::listen(listener.as_raw_fd(), 1) }, 0);
        let holders = [
            UnixStream::connect(&path).expect("holder a"),
            UnixStream::connect(&path).expect("holder b"),
        ];
        Some(Self {
            path,
            _dir: dir,
            _listener: listener,
            _holders: holders,
        })
    }
}

#[cfg(target_os = "linux")]
fn connect_budget() -> Duration {
    SOCKET_TIMEOUT + Duration::from_millis(750)
}

#[cfg(target_os = "linux")]
fn assert_connect_times_out(send: impl FnOnce() -> Result<(), String>) {
    let start = Instant::now();
    let err = send().expect_err("wedged accept must not succeed");
    assert!(
        start.elapsed() <= connect_budget(),
        "expected return near {:?}, got {:?}; err={err}",
        SOCKET_TIMEOUT,
        start.elapsed()
    );
    assert!(
        err.contains("timed out"),
        "expected connect timeout, got: {err}"
    );
}

#[cfg(target_os = "linux")]
fn assert_returns_within_budget(send: impl FnOnce()) {
    let start = Instant::now();
    send();
    assert!(
        start.elapsed() <= connect_budget(),
        "send_request must also bound connect"
    );
}
