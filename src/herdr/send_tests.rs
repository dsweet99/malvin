//! Unit tests for herdr socket send helpers.

use super::{classify_reply, send_request, send_request_checked, SOCKET_TIMEOUT};
use serde_json::json;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn send_request_writes_ndjson_line_to_unix_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("t.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
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
    assert!(send_request_checked(
        std::path::Path::new("/no/such/herdr.sock"),
        &json!({})
    )
    .is_err());
}

#[test]
fn classify_reply_detects_herdr_error_json() {
    assert!(classify_reply(br#"{"id":"1","error":{"code":"x","message":"no"}}"#).is_err());
    assert!(classify_reply(br#"{"result":{"type":"ok"}}"#).is_ok());
    assert!(classify_reply(b"").is_ok());
}
