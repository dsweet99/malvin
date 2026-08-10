//! Tests for [`super::LocalOpenAiServer`].

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use super::*;
use crate::local_llm::LocalCompletionEngine;

fn post_chat_completions(base_url: &str, body: &[u8]) -> String {
    let host = base_url
        .trim_start_matches("http://")
        .trim_end_matches("/v1");
    let req = format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let mut stream = TcpStream::connect(host).expect("connect");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("rt");
    stream.write_all(req.as_bytes()).expect("headers");
    stream.write_all(body).expect("body");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");
    resp
}

#[test]
fn scripted_server_answers_chat_completions() {
    let engine = LocalCompletionEngine::scripted_ok("qwen35_9b_q4", "pong");
    let server = LocalOpenAiServer::start(engine).expect("start");
    std::thread::sleep(Duration::from_millis(30));
    let body = br#"{"model":"local/qwen35_9b_q4","stream":true,"messages":[{"role":"user","content":"ping"}]}"#;
    let resp = post_chat_completions(&server.base_url, body);
    assert!(resp.contains("pong"), "{resp}");
    assert!(resp.contains("[DONE]"), "{resp}");
}

#[test]
fn parse_messages_maps_developer_to_system() {
    let body = json!({
        "messages": [
            { "role": "developer", "content": "sys" },
            { "role": "user", "content": [{ "type": "text", "text": "hi" }] }
        ]
    });
    let msgs = parse_messages(&body).expect("msgs");
    assert_eq!(msgs[0].role, ChatRole::System);
    assert_eq!(msgs[1].content, "hi");
}
