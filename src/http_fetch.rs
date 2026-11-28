use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) struct HttpRequest<'a> {
    pub(crate) url: &'a str,
    pub(crate) headers: &'a [(&'a str, String)],
    pub(crate) json_body: Option<&'a serde_json::Value>,
    pub(crate) timeout: Duration,
}

impl<'a> HttpRequest<'a> {
    pub(crate) const fn get(url: &'a str, timeout: Duration) -> Self {
        Self {
            url,
            headers: &[],
            json_body: None,
            timeout,
        }
    }
}

fn header_block(req: &HttpRequest<'_>) -> String {
    let mut block = String::new();
    for (name, value) in req.headers {
        block.push_str(name);
        block.push_str(": ");
        block.push_str(value);
        block.push('\n');
    }
    block
}

fn header_stdin(req: &HttpRequest<'_>) -> Stdio {
    if req.headers.is_empty() {
        Stdio::null()
    } else {
        Stdio::piped()
    }
}

fn curl_command(req: &HttpRequest<'_>) -> Command {
    let mut cmd = Command::new("curl");
    cmd.args(["-sS", "-f", "-L", "--max-time"])
        .arg(req.timeout.as_secs_f64().max(0.1).to_string())
        .args(["-H", "Accept: application/json"]);
    if !req.headers.is_empty() {
        cmd.args(["-H", "@-"]);
    }
    if let Some(body) = req.json_body {
        cmd.args(["-H", "Content-Type: application/json", "--data-binary"])
            .arg(body.to_string());
    }
    cmd.arg(req.url)
        .stdin(header_stdin(req))
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd
}

#[must_use]
pub(crate) fn fetch_text(req: &HttpRequest<'_>) -> Option<String> {
    let mut child = curl_command(req).spawn().ok()?;
    if let Some(mut stdin) = child.stdin.take() {
        let written = stdin.write_all(header_block(req).as_bytes());
        drop(stdin);
        if written.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    }
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(test)]
pub(crate) fn serve_once(body: &'static str) -> String {
    use std::io::Read;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    std::thread::spawn(move || {
        if let Ok((mut socket, _)) = listener.accept() {
            let _ = socket.read(&mut [0_u8; 2048]);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = socket.write_all(response.as_bytes());
        }
    });
    format!("http://{addr}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serve_once_capturing_request() -> (String, std::sync::mpsc::Receiver<String>) {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Ok((mut socket, _)) = listener.accept() {
                let mut buf = [0_u8; 4096];
                let n = socket.read(&mut buf).unwrap_or(0);
                let _ = tx.send(String::from_utf8_lossy(&buf[..n]).into_owned());
                let _ = socket.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                );
            }
        });
        (format!("http://{addr}/h"), rx)
    }

    #[test]
    fn header_values_stay_off_the_curl_command_line_but_reach_the_server() {
        let secret = "sk-test-SECRET-value";
        let headers = [("Authorization", format!("Bearer {secret}"))];
        let (url, rx) = serve_once_capturing_request();
        let req = HttpRequest {
            url: &url,
            headers: &headers,
            json_body: None,
            timeout: Duration::from_secs(5),
        };
        let cmd = curl_command(&req);
        assert!(
            cmd.get_args()
                .all(|arg| !arg.to_string_lossy().contains(secret)),
            "secret leaked into argv"
        );
        assert_eq!(fetch_text(&req).as_deref(), Some("{}"));
        let request = rx.recv_timeout(Duration::from_secs(5)).expect("request");
        assert!(request.contains(&format!("Authorization: Bearer {secret}\r\n")));
    }
    #[test]
    fn fetch_text_reads_local_server_body_and_fetch_text_is_none_when_nothing_listens() {
        {
            let base = serve_once(r#"{"ok":true}"#);
            let url = format!("{base}/x");
            let body = fetch_text(&HttpRequest::get(&url, Duration::from_secs(2))).expect("body");
            assert_eq!(body, r#"{"ok":true}"#);
        }
        {
            let url = "http://127.0.0.1:9/nothing";
            assert!(fetch_text(&HttpRequest::get(url, Duration::from_millis(500))).is_none());
        }
    }
}
