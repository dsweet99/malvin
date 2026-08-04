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
    let _ = send_request_checked(socket_path, request);
}

/// Send and classify the reply when one arrives (ok / herdr error / I/O error).
pub(crate) fn send_request_checked(socket_path: &Path, request: &Value) -> Result<(), String> {
    let mut line = serde_json::to_string(request).map_err(|e| e.to_string())?;
    line.push('\n');
    let mut stream = open_timed_stream(socket_path)?;
    stream
        .write_all(line.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut buf = [0_u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    classify_reply(&buf[..n])
}

fn open_timed_stream(socket_path: &Path) -> Result<UnixStream, String> {
    let stream = UnixStream::connect(socket_path).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|e| e.to_string())?;
    Ok(stream)
}

fn classify_reply(bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Ok(());
    }
    let text = String::from_utf8_lossy(bytes);
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        return Ok(());
    }
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return Ok(());
    };
    if let Some(err) = v.get("error") {
        let code = err.get("code").and_then(Value::as_str).unwrap_or("error");
        let msg = err.get("message").and_then(Value::as_str).unwrap_or("");
        return Err(format!("{code}: {msg}"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "send_tests.rs"]
mod send_tests;
