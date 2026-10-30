use super::{
    MAX_REPLY_BYTES, SOCKET_TIMEOUT, read_reply_bytes, read_reply_chunk, send_request_checked,
};
use serde_json::json;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::time::{Duration, Instant};

fn request_with_reply(reply: Vec<u8>, chunk_size: usize) -> Result<(), String> {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("reply.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = thread::spawn(move || {
        let (mut conn, _) = listener.accept().expect("accept");
        conn.set_read_timeout(Some(Duration::from_secs(2)))
            .expect("server timeout");
        let mut request = String::new();
        BufReader::new(&conn)
            .read_line(&mut request)
            .expect("request");
        for chunk in reply.chunks(chunk_size) {
            if conn.write_all(chunk).is_err() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        // Keep the connection open: a newline must complete the reply without EOF.
        let _ = conn.read_to_end(&mut Vec::new());
    });
    let result = send_request_checked(&sock, &json!({"id":"t","method":"ping","params":{}}));
    server.join().expect("server");
    result
}

#[test]
fn fragmented_reply_accepts_split_utf8_and_stops_at_first_newline() {
    let reply = "{\"result\":\"é\"}\ninvalid second frame\n"
        .as_bytes()
        .to_vec();
    assert_eq!(request_with_reply(reply, 1), Ok(()));
}

#[test]
fn reply_larger_than_one_read_is_accepted() {
    let reply = format!("{{\"result\":\"{}\"}}\n", "x".repeat(8192)).into_bytes();
    assert_eq!(request_with_reply(reply, 2048), Ok(()));
}

#[test]
fn fragmented_error_reply_preserves_server_diagnostic() {
    let reply = b"{\"error\":{\"code\":\"denied\",\"message\":\"no\"}}\n".to_vec();
    assert_eq!(request_with_reply(reply, 8), Err("denied: no".into()));
}

#[test]
fn eof_terminates_reply_and_partial_json_is_rejected() {
    for reply in [b"{\"result\":{}}".as_slice(), b"{\"result\":".as_slice()] {
        let (mut client, mut server) = UnixStream::pair().expect("pair");
        server.write_all(reply).expect("write");
        drop(server);
        let bytes = read_reply_bytes(&mut client).expect("EOF frame");
        assert_eq!(super::classify_reply(&bytes).is_ok(), reply.ends_with(b"}"));
    }
}

#[test]
fn reply_size_limit_is_enforced() {
    let reply = vec![b'x'; MAX_REPLY_BYTES + 1];
    assert_eq!(
        request_with_reply(reply, 4096),
        Err("herdr reply too large".into())
    );
}

#[test]
fn reply_at_size_limit_is_accepted() {
    let reply = format!("{{\"result\":\"{}\"}}\n", "x".repeat(MAX_REPLY_BYTES - 14)).into_bytes();
    assert_eq!(reply.len(), MAX_REPLY_BYTES);
    assert_eq!(request_with_reply(reply, 4096), Ok(()));
}

#[test]
fn expired_read_deadline_returns_timeout() {
    let (mut client, _server) = UnixStream::pair().expect("pair");
    let error = read_reply_chunk(&mut client, &mut [0; 8], Instant::now() - SOCKET_TIMEOUT)
        .expect_err("expired deadline");
    assert!(error.contains("timed out"), "{error}");
}

#[test]
fn trickling_reply_cannot_extend_total_read_timeout() {
    let (mut client, mut server) = UnixStream::pair().expect("pair");
    let writer = thread::spawn(move || {
        for _ in 0..20 {
            if server.write_all(b" ").is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
    });
    let start = Instant::now();
    let error = read_reply_bytes(&mut client).expect_err("unterminated reply");
    assert!(error.contains("timed out"), "{error}");
    assert!(start.elapsed() < SOCKET_TIMEOUT + Duration::from_millis(500));
    drop(client);
    writer.join().expect("writer");
}
