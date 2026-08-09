//! Minimal HTTP/1.1 request read helpers for the local OpenAI-compatible sidecar.

use std::io::Read;
use std::net::TcpStream;

pub(super) struct HttpRequest {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

pub(super) fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut buf = read_until_headers(stream)?;
    let header_end = find_header_end(&buf).ok_or_else(|| "incomplete HTTP headers".to_string())?;
    let (method, path, content_length) = parse_request_head(&buf[..header_end])?;
    let mut body = buf.split_off(header_end + 4);
    read_body_remainder(stream, &mut body, content_length)?;
    body.truncate(content_length);
    Ok(HttpRequest { method, path, body })
}

fn read_until_headers(stream: &mut TcpStream) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        let n = stream.read(&mut chunk).map_err(|e| format!("read: {e}"))?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if find_header_end(&buf).is_some() {
            break;
        }
        if buf.len() > 1024 * 1024 {
            return Err("request headers too large".into());
        }
    }
    Ok(buf)
}

fn parse_request_head(header: &[u8]) -> Result<(String, String, usize), String> {
    let header = std::str::from_utf8(header).map_err(|e| e.to_string())?;
    let mut lines = header.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let content_length = content_length_from_headers(lines);
    Ok((method, path, content_length))
}

fn content_length_from_headers<'a>(mut lines: impl Iterator<Item = &'a str>) -> usize {
    lines
        .find_map(|line| {
            let (k, v) = line.split_once(':')?;
            if k.eq_ignore_ascii_case("content-length") {
                v.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

fn read_body_remainder(
    stream: &mut TcpStream,
    body: &mut Vec<u8>,
    content_length: usize,
) -> Result<(), String> {
    let mut chunk = [0_u8; 4096];
    while body.len() < content_length {
        let n = stream
            .read(&mut chunk)
            .map_err(|e| format!("read body: {e}"))?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }
    Ok(())
}

pub(super) fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_header_end_locates_separator() {
        assert_eq!(find_header_end(b"GET / HTTP/1.1\r\n\r\n"), Some(14));
        assert!(find_header_end(b"nope").is_none());
    }
}
