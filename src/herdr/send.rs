//! Short-timeout Unix-socket NDJSON transport for herdr.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use serde_json::Value;

/// Connect/read budget matching the cursor herdr hook (~0.5s).
pub const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);

/// Best-effort send; never propagates I/O errors to the caller.
pub fn send_request(socket_path: &Path, request: &Value) {
    let _ = send_request_result(socket_path, request);
}

/// Best-effort send with one immediate retry on failure (teardown needs this).
pub(crate) fn send_request_retry(socket_path: &Path, request: &Value) -> bool {
    send_request_result(socket_path, request).is_ok()
        || send_request_result(socket_path, request).is_ok()
}

pub(crate) fn send_request_result(socket_path: &Path, request: &Value) -> Result<(), ()> {
    let mut line = serde_json::to_string(request).map_err(|_| ())?;
    line.push('\n');
    let mut stream = UnixStream::connect(socket_path).map_err(|_| ())?;
    stream.set_read_timeout(Some(SOCKET_TIMEOUT)).map_err(|_| ())?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT)).map_err(|_| ())?;
    stream.write_all(line.as_bytes()).map_err(|_| ())?;
    let mut buf = [0_u8; 4096];
    let _ = stream.read(&mut buf);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{send_request, send_request_result, send_request_retry, SOCKET_TIMEOUT};
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
        let _ = send_request_result;
        let _ = send_request_retry;
    }

    #[test]
    fn send_request_swallows_missing_socket() {
        send_request(std::path::Path::new("/no/such/herdr.sock"), &json!({}));
        assert!(!send_request_retry(
            std::path::Path::new("/no/such/herdr.sock"),
            &json!({})
        ));
    }
}
