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

fn curl_command(req: &HttpRequest<'_>) -> Command {
    let mut cmd = Command::new("curl");
    cmd.args(["-sS", "-f", "-L", "--max-time"])
        .arg(req.timeout.as_secs_f64().max(0.1).to_string())
        .args(["-H", "Accept: application/json"]);
    for (name, value) in req.headers {
        cmd.arg("-H").arg(format!("{name}: {value}"));
    }
    if let Some(body) = req.json_body {
        cmd.args(["-H", "Content-Type: application/json", "--data-binary"])
            .arg(body.to_string());
    }
    cmd.arg(req.url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd
}

#[must_use]
pub(crate) fn fetch_text(req: &HttpRequest<'_>) -> Option<String> {
    let output = curl_command(req).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(test)]
pub(crate) fn serve_once(body: &'static str) -> String {
    use std::io::{Read, Write};
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

    #[test]
    fn fetch_text_reads_local_server_body() {
        let base = serve_once(r#"{"ok":true}"#);
        let url = format!("{base}/x");
        let body = fetch_text(&HttpRequest::get(&url, Duration::from_secs(2))).expect("body");
        assert_eq!(body, r#"{"ok":true}"#);
    }

    #[test]
    fn fetch_text_is_none_when_nothing_listens() {
        let url = "http://127.0.0.1:9/nothing";
        assert!(fetch_text(&HttpRequest::get(url, Duration::from_millis(500))).is_none());
    }
}
