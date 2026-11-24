use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

pub const SOCKET_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_REPLY_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HerdrError {
    pub code: Option<String>,
    pub message: String,
}

impl HerdrError {
    pub(crate) fn has_code(&self, code: &str) -> bool {
        self.code.as_deref() == Some(code)
    }
}

impl From<String> for HerdrError {
    fn from(message: String) -> Self {
        Self {
            code: None,
            message,
        }
    }
}

impl std::fmt::Display for HerdrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.code {
            Some(code) => write!(f, "{code}: {}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

#[cfg(test)]
pub fn send_request(socket_path: &Path, request: &Value) {
    let _ = send_request_checked(socket_path, request);
}

pub(crate) fn send_request_checked(socket_path: &Path, request: &Value) -> Result<(), HerdrError> {
    send_request_reply(socket_path, request).map(|_| ())
}

pub(crate) fn send_request_reply(socket_path: &Path, request: &Value) -> Result<Value, HerdrError> {
    let mut line = serde_json::to_string(request).map_err(|e| e.to_string())?;
    line.push('\n');
    let mut stream = open_timed_stream(socket_path)?;
    stream
        .write_all(line.as_bytes())
        .map_err(|e| HerdrError::from(e.to_string()))?;
    classify_reply(&read_reply_bytes(&mut stream)?)
}

pub(crate) fn send_request_with_retry(
    socket_path: &Path,
    request: &Value,
) -> Result<(), HerdrError> {
    if send_request_checked(socket_path, request).is_ok() {
        return Ok(());
    }
    send_request_checked(socket_path, request)
}

fn read_reply_bytes(stream: &mut UnixStream) -> Result<Vec<u8>, String> {
    let deadline = Instant::now() + SOCKET_TIMEOUT;
    let mut reply = Vec::new();
    let mut buf = [0_u8; 4096];
    loop {
        let n = read_reply_chunk(stream, &mut buf, deadline)?;
        let newline = buf[..n].iter().position(|&byte| byte == b'\n');
        let end = newline.map_or(n, |index| index + 1);
        if reply.len() + end > MAX_REPLY_BYTES {
            return Err("herdr reply too large".into());
        }
        reply.extend_from_slice(&buf[..end]);
        if n == 0 || newline.is_some() {
            return Ok(reply);
        }
    }
}

fn read_reply_chunk(
    stream: &mut UnixStream,
    buf: &mut [u8],
    deadline: Instant,
) -> Result<usize, String> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| "herdr read timed out".to_string())?;
    let _ = stream.set_read_timeout(Some(remaining));
    match stream.read(buf) {
        Ok(n) => Ok(n),
        Err(e)
            if e.kind() == std::io::ErrorKind::TimedOut
                || e.kind() == std::io::ErrorKind::WouldBlock =>
        {
            Err(format!("herdr read timed out: {e}"))
        }
        Err(e) => Err(format!("herdr read: {e}")),
    }
}

fn first_nonempty_line(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("herdr reply empty".into());
    }
    let text = String::from_utf8_lossy(bytes);
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() {
        Err("herdr reply empty".into())
    } else {
        Ok(line.to_string())
    }
}

fn herdr_json_error(v: &Value) -> Option<HerdrError> {
    let err = v.get("error")?;
    let code = err.get("code").and_then(Value::as_str).unwrap_or("error");
    let message = err.get("message").and_then(Value::as_str).unwrap_or("");
    Some(HerdrError {
        code: Some(code.to_string()),
        message: message.to_string(),
    })
}

fn classify_reply(bytes: &[u8]) -> Result<Value, HerdrError> {
    let line = first_nonempty_line(bytes)?;
    let v =
        serde_json::from_str::<Value>(&line).map_err(|e| format!("herdr reply not json: {e}"))?;
    if let Some(error) = herdr_json_error(&v) {
        return Err(error);
    }
    v.get("result")
        .cloned()
        .ok_or_else(|| HerdrError::from("herdr reply missing result".to_string()))
}

fn open_timed_stream(socket_path: &Path) -> Result<UnixStream, String> {
    let stream = connect_with_timeout(socket_path, SOCKET_TIMEOUT)?;
    stream
        .set_read_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(SOCKET_TIMEOUT))
        .map_err(|e| e.to_string())?;
    Ok(stream)
}

fn connect_with_timeout(socket_path: &Path, timeout: Duration) -> Result<UnixStream, String> {
    let path = PathBuf::from(socket_path);
    let (tx, rx) = mpsc::channel();
    thread::Builder::new()
        .name("herdr-connect".into())
        .spawn(move || {
            let _ = tx.send(UnixStream::connect(path));
        })
        .map_err(|e| e.to_string())?;
    match rx.recv_timeout(timeout) {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(e)) => Err(e.to_string()),
        Err(mpsc::RecvTimeoutError::Timeout) => Err("herdr connect timed out".into()),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err("herdr connect worker died".into()),
    }
}

#[cfg(test)]
#[path = "send_tests.rs"]
mod send_tests;

#[cfg(test)]
#[path = "send_reply_tests.rs"]
mod send_reply_tests;
