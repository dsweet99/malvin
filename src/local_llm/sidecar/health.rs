//! Health probe for the local sidecar HTTP endpoint.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

const HEALTH_WAIT: Duration = Duration::from_secs(120);
const HEALTH_POLL: Duration = Duration::from_millis(250);

pub(super) fn wait_for_health(base_url: &str) -> Result<(), String> {
    let (host, port) = parse_loopback_base_url(base_url)?;
    let deadline = Instant::now() + HEALTH_WAIT;
    while Instant::now() < deadline {
        if http_get_ok(&host, port, "/v1/models") {
            return Ok(());
        }
        thread::sleep(HEALTH_POLL);
    }
    Err(format!("timed out waiting for http://{host}:{port}/v1/models"))
}

fn parse_loopback_base_url(base_url: &str) -> Result<(String, u16), String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let without_scheme = trimmed
        .strip_prefix("http://")
        .ok_or_else(|| format!("expected http:// base url, got {base_url}"))?;
    let authority = without_scheme
        .split('/')
        .next()
        .ok_or_else(|| format!("invalid base url {base_url}"))?;
    let (host, port_str) = authority
        .rsplit_once(':')
        .ok_or_else(|| format!("missing port in {base_url}"))?;
    let port: u16 = port_str
        .parse()
        .map_err(|e| format!("invalid port in {base_url}: {e}"))?;
    Ok((host.to_string(), port))
}

fn http_get_ok(host: &str, port: u16, path: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect((host, port)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let req = format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let mut buf = [0_u8; 128];
    let Ok(n) = stream.read(&mut buf) else {
        return false;
    };
    let head = String::from_utf8_lossy(&buf[..n]);
    head.starts_with("HTTP/1.1 200") || head.starts_with("HTTP/1.0 200")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_loopback_base_url_reads_host_port() {
        let (host, port) = parse_loopback_base_url("http://127.0.0.1:1234/v1").expect("ok");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 1234);
        assert!(parse_loopback_base_url("https://x").is_err());
    }

    #[test]
    fn http_get_ok_false_when_nothing_listens() {
        assert!(!http_get_ok("127.0.0.1", 1, "/v1/models"));
    }

    #[test]
    fn wait_for_health_rejects_bad_url() {
        assert!(wait_for_health("not-a-url").is_err());
    }
}
