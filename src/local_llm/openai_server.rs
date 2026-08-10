//! Minimal OpenAI-compatible `/v1/chat/completions` server for Prime local GGUF models.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde_json::{json, Value};

use crate::llm_transport::{ChatMessage, ChatRole};
use crate::local_llm::LocalCompletionEngine;

use super::openai_http::{read_http_request, HttpRequest};

/// Background localhost server wrapping [`LocalCompletionEngine`].
pub struct LocalOpenAiServer {
    shutdown: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    /// Base URL including `/v1` (for Prime `models.json` `baseUrl`).
    pub base_url: String,
}

impl LocalOpenAiServer {
    /// Bind `127.0.0.1:0` and serve streaming chat completions until dropped.
    ///
    /// # Errors
    ///
    /// Returns an error when the listener cannot bind.
    pub fn start(engine: LocalCompletionEngine) -> Result<Self, String> {
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| format!("local openai bind: {e}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("local openai nonblocking: {e}"))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("local openai addr: {e}"))?
            .port();
        let shutdown = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&shutdown);
        let engine = Arc::new(engine);
        let join = thread::spawn(move || accept_loop(listener, engine, flag));
        Ok(Self {
            shutdown,
            join: Some(join),
            base_url: format!("http://127.0.0.1:{port}/v1"),
        })
    }
}

impl Drop for LocalOpenAiServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn accept_loop(listener: TcpListener, engine: Arc<LocalCompletionEngine>, shutdown: Arc<AtomicBool>) {
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                let engine = Arc::clone(&engine);
                thread::spawn(move || {
                    let _ = handle_connection(stream, &engine);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(15));
            }
            Err(_) => break,
        }
    }
}

fn handle_connection(mut stream: TcpStream, engine: &LocalCompletionEngine) -> Result<(), String> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(120)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(120)));
    match read_http_request(&mut stream) {
        Ok(req) => respond_to_request(&mut stream, engine, &req),
        Err(e) => {
            let payload = json!({ "error": { "message": e } }).to_string();
            write_response(&mut stream, 400, "application/json", payload.as_bytes())
        }
    }
}

fn respond_to_request(
    stream: &mut TcpStream,
    engine: &LocalCompletionEngine,
    req: &HttpRequest,
) -> Result<(), String> {
    if !req.path.contains("/chat/completions") {
        return write_response(stream, 404, "text/plain", b"not found");
    }
    if req.method != "POST" {
        return write_response(stream, 405, "text/plain", b"method not allowed");
    }
    let body: Value =
        serde_json::from_slice(&req.body).map_err(|e| format!("json body: {e}"))?;
    let messages = parse_messages(&body)?;
    match block_on_complete(engine, &messages).0 {
        Ok(resp) => write_sse_completion(stream, &resp.content),
        Err(e) => {
            let payload = json!({ "error": { "message": e.to_string() } }).to_string();
            write_response(stream, 500, "application/json", payload.as_bytes())
        }
    }
}

fn block_on_complete(
    engine: &LocalCompletionEngine,
    messages: &[ChatMessage],
) -> (
    Result<crate::llm_transport::CompletionResponse, crate::llm_transport::TransportError>,
    crate::llm_transport::HttpExchangeMeta,
) {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt.block_on(engine.complete(messages)),
        Err(e) => (
            Err(crate::llm_transport::TransportError::Engine(format!(
                "tokio runtime: {e}"
            ))),
            crate::llm_transport::HttpExchangeMeta {
                status: Some(500),
                body: None,
            },
        ),
    }
}

pub(super) fn parse_messages(body: &Value) -> Result<Vec<ChatMessage>, String> {
    let arr = body
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| "missing messages".to_string())?;
    let mut out = Vec::with_capacity(arr.len());
    for msg in arr {
        out.push(chat_message_from_json(msg));
    }
    Ok(out)
}

fn chat_message_from_json(msg: &Value) -> ChatMessage {
    let role = msg.get("role").and_then(Value::as_str).unwrap_or("user");
    ChatMessage {
        role: match role {
            "system" | "developer" => ChatRole::System,
            "assistant" => ChatRole::Assistant,
            _ => ChatRole::User,
        },
        content: message_content(msg),
    }
}

pub(super) fn message_content(msg: &Value) -> String {
    match msg.get("content") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(parts)) => parts
            .iter()
            .filter_map(|p| p.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn write_sse_completion(stream: &mut TcpStream, content: &str) -> Result<(), String> {
    let id = "malvin-local";
    let chunk = json!({
        "id": id,
        "object": "chat.completion.chunk",
        "choices": [{
            "index": 0,
            "delta": { "role": "assistant", "content": content },
            "finish_reason": null
        }]
    });
    let done = json!({
        "id": id,
        "object": "chat.completion.chunk",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": "stop"
        }]
    });
    let body = format!("data: {chunk}\n\ndata: {done}\n\ndata: [DONE]\n\n");
    write_response(stream, 200, "text/event-stream", body.as_bytes())
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .map_err(|e| format!("write header: {e}"))?;
    stream
        .write_all(body)
        .map_err(|e| format!("write body: {e}"))?;
    Ok(())
}

#[cfg(test)]
#[path = "openai_server_tests.rs"]
mod openai_server_tests;

#[cfg(test)]
#[path = "openai_server_kiss_cov_tests.rs"]
mod openai_server_kiss_cov_tests;
