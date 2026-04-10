//! Stdout JSON-RPC line processing for ACP.
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

use super::{ResponseTx, transport::write_rpc_line};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{Mutex, Notify};
use tracing::{debug, error, info, trace, warn};

/// Clears busy / trace when the JSON-RPC response for a `session/prompt` request is processed.
pub(crate) struct PromptRpcCleanup {
    pub busy: Arc<AtomicBool>,
    pub trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    pub prompt_rpc_id: Arc<AtomicU64>,
    /// When set (UI lane only), [`Self::clear_if_prompt_response`] notifies waiters when `busy` clears.
    pub idle_notify: Option<Arc<Notify>>,
}

impl PromptRpcCleanup {
    pub async fn clear_if_prompt_response(&self, id: u64) {
        let expected = self.prompt_rpc_id.load(Ordering::SeqCst);
        if expected != 0 && expected == id {
            self.prompt_rpc_id.store(0, Ordering::SeqCst);
            self.busy.store(false, Ordering::SeqCst);
            *self.trace_writer.lock().await = None;
            if let Some(n) = &self.idle_notify {
                n.notify_waiters();
            }
        }
    }
}

enum SessionUpdateChunkKind {
    Message,
    Thought,
}

/// Chunk text coalescing for **verbose** logs and **JSONL traces**: append until this many Unicode scalars,
/// a newline run, or a non-chunk line (JSON-RPC response, `tool_call`, etc.) triggers a flush.
pub(crate) const ACP_VERBOSE_COALESCE_MAX: usize = 125;

/// Append `chunk` to `buf`, pushing completed coalesce segments into `emissions` (no newlines in segments).
fn coalesce_append_chunk(buf: &mut String, chunk: &str, emissions: &mut Vec<String>) {
    let mut pos = 0usize;
    let b = chunk.as_bytes();
    while pos < b.len() {
        if let Some(rel) = b[pos..].iter().position(|&c| c == b'\n') {
            let end = pos + rel;
            buf.push_str(&chunk[pos..end]);
            coalesce_flush_cap(buf, emissions);
            coalesce_flush_nonempty(buf, emissions);
            pos = end;
            while pos < b.len() && b[pos] == b'\n' {
                pos += 1;
            }
        } else {
            buf.push_str(&chunk[pos..]);
            coalesce_flush_cap(buf, emissions);
            break;
        }
    }
}

fn coalesce_char_boundary_at(s: &str, n_chars: usize) -> usize {
    s.char_indices()
        .nth(n_chars)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn coalesce_flush_cap(buf: &mut String, emissions: &mut Vec<String>) {
    while buf.chars().count() >= ACP_VERBOSE_COALESCE_MAX {
        let end = coalesce_char_boundary_at(buf, ACP_VERBOSE_COALESCE_MAX);
        emissions.push(buf.drain(..end).collect());
    }
}

fn coalesce_flush_nonempty(buf: &mut String, emissions: &mut Vec<String>) {
    if !buf.is_empty() {
        emissions.push(std::mem::take(buf));
    }
}

#[derive(Default)]
struct VerboseIoCoalescer {
    message: String,
    thought: String,
}

impl VerboseIoCoalescer {
    fn feed(&mut self, kind: SessionUpdateChunkKind, chunk: &str) {
        match kind {
            SessionUpdateChunkKind::Message => Self::feed_buf(&mut self.message, chunk, "acp message"),
            SessionUpdateChunkKind::Thought => Self::feed_buf(&mut self.thought, chunk, "acp thought"),
        }
    }

    fn flush_all(&mut self) {
        Self::flush_if_nonempty(&mut self.message, "acp message");
        Self::flush_if_nonempty(&mut self.thought, "acp thought");
    }

    fn feed_buf(buf: &mut String, chunk: &str, label: &'static str) {
        let mut emissions = Vec::new();
        coalesce_append_chunk(buf, chunk, &mut emissions);
        for piece in emissions {
            info!(target: "malvin::acp::io", "{} {}", label, piece);
        }
    }

    fn flush_if_nonempty(buf: &mut String, label: &'static str) {
        if !buf.is_empty() {
            let piece = std::mem::take(buf);
            info!(target: "malvin::acp::io", "{} {}", label, piece);
        }
    }
}

/// `session/update` streaming chunks (`agent_message_chunk`, `agent_thought_chunk`).
fn session_update_chunk_parts(v: &Value) -> Option<(SessionUpdateChunkKind, String)> {
    if v.get("method").and_then(Value::as_str) != Some("session/update") {
        return None;
    }
    let update = v.pointer("/params/update")?;
    let kind = match update.get("sessionUpdate").and_then(Value::as_str)? {
        "agent_message_chunk" => SessionUpdateChunkKind::Message,
        "agent_thought_chunk" => SessionUpdateChunkKind::Thought,
        _ => return None,
    };
    let text = update
        .pointer("/content/text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    Some((kind, text))
}

fn patch_session_update_chunk_text(template: Value, merged_text: &str) -> Option<Value> {
    let mut v = template;
    let content = v.pointer_mut("/params/update/content")?.as_object_mut()?;
    content.insert("text".to_string(), Value::String(merged_text.to_string()));
    Some(v)
}

#[derive(Default)]
struct TraceChunkCoalescer {
    message: String,
    thought: String,
    message_tpl: Option<Value>,
    thought_tpl: Option<Value>,
}

impl TraceChunkCoalescer {
    fn feed(
        &mut self,
        kind: SessionUpdateChunkKind,
        chunk: &str,
        line_value: &Value,
    ) -> Vec<String> {
        let (buf, tpl) = match kind {
            SessionUpdateChunkKind::Message => (&mut self.message, &mut self.message_tpl),
            SessionUpdateChunkKind::Thought => (&mut self.thought, &mut self.thought_tpl),
        };
        if !chunk.is_empty() && buf.is_empty() {
            *tpl = Some(line_value.clone());
        }
        let mut emissions = Vec::new();
        coalesce_append_chunk(buf, chunk, &mut emissions);
        let mut out = Vec::new();
        for piece in emissions {
            if let Some(t) = tpl.as_ref()
                && let Some(patched) = patch_session_update_chunk_text(t.clone(), &piece)
                && let Ok(s) = serde_json::to_string(&patched)
            {
                out.push(s);
            }
        }
        if buf.is_empty() {
            *tpl = None;
        }
        out
    }

    fn flush_all(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        Self::flush_stream(&mut self.message, &mut self.message_tpl, &mut out);
        Self::flush_stream(&mut self.thought, &mut self.thought_tpl, &mut out);
        out
    }

    fn flush_stream(buf: &mut String, tpl: &mut Option<Value>, out: &mut Vec<String>) {
        if buf.is_empty() {
            return;
        }
        let piece = std::mem::take(buf);
        if let Some(t) = tpl.take()
            && let Some(patched) = patch_session_update_chunk_text(t, &piece)
            && let Ok(s) = serde_json::to_string(&patched)
        {
            out.push(s);
        }
    }
}

async fn trace_file_write_line(f: &mut tokio::fs::File, line: &str) {
    if let Err(e) = f.write_all(line.as_bytes()).await {
        warn!(error = %e, "trace write failed");
    } else if let Err(e) = f.write_all(b"\n").await {
        warn!(error = %e, "trace newline failed");
    }
}

async fn write_trace_line_coalesced(
    raw_line: &str,
    trace_file: &mut tokio::fs::File,
    coalesce: &mut TraceChunkCoalescer,
    parsed: &Option<Value>,
) {
    if let Some(v) = parsed
        && let Some((kind, text)) = session_update_chunk_parts(v)
    {
        for tl in coalesce.feed(kind, text.as_str(), v) {
            trace_file_write_line(trace_file, &tl).await;
        }
        return;
    }
    for tl in coalesce.flush_all() {
        trace_file_write_line(trace_file, &tl).await;
    }
    trace_file_write_line(trace_file, raw_line).await;
}

async fn reader_loop_verbose_and_trace_line(
    line: &str,
    acp_verbose: bool,
    trace_writer: &Arc<Mutex<Option<tokio::fs::File>>>,
    verbose_coalesce: &mut VerboseIoCoalescer,
    trace_coalesce: &mut TraceChunkCoalescer,
) {
    let tracing = {
        let g = trace_writer.lock().await;
        g.is_some()
    };
    let parsed: Option<Value> = if acp_verbose || tracing {
        serde_json::from_str(line).ok()
    } else {
        None
    };

    if acp_verbose {
        match parsed.as_ref().and_then(session_update_chunk_parts) {
            Some((kind, text)) => {
                verbose_coalesce.feed(kind, text.as_str());
            }
            None => {
                verbose_coalesce.flush_all();
                info!(
                    target: "malvin::acp::io",
                    line = %line,
                    "acp message"
                );
            }
        }
    }

    let mut g = trace_writer.lock().await;
    if let Some(ref mut f) = *g {
        write_trace_line_coalesced(line, f, trace_coalesce, &parsed).await;
    }
}

/// JSON-RPC 2.0 allows `id` as string or number; map nonnegative integers (and decimal strings) to
/// `u64` for pending lookup.
/// Correlation id for `session/request_permission`: JSON-RPC root `id`, or `params.id` /
/// `params.requestId` when the server nests it (some peers omit the top-level field).
fn request_permission_correlation_id(msg: &Value) -> Option<&Value> {
    if let Some(id) = msg.get("id")
        && !id.is_null()
    {
        return Some(id);
    }
    let params = msg.get("params")?;
    let obj = params.as_object()?;
    if let Some(id) = obj.get("id")
        && !id.is_null()
    {
        return Some(id);
    }
    let id = obj.get("requestId")?;
    if id.is_null() {
        None
    } else {
        Some(id)
    }
}

fn jsonrpc_response_id_as_u64(id_v: &Value) -> Option<u64> {
    if let Some(n) = id_v.as_u64() {
        return Some(n);
    }
    if let Some(n) = id_v.as_i64()
        && n >= 0
    {
        return Some(n as u64);
    }
    id_v.as_str()?.parse::<u64>().ok()
}

pub(crate) async fn dispatch_response(
    msg: &Value,
    pending: &Arc<Mutex<HashMap<u64, ResponseTx>>>,
    prompt_cleanup: Option<&PromptRpcCleanup>,
) -> bool {
    let Some(id_v) = msg.get("id") else {
        return false;
    };
    let Some(id) = jsonrpc_response_id_as_u64(id_v) else {
        warn!(id = ?id_v, "acp response id is not a nonnegative integer or decimal string");
        return false;
    };
    let Some(tx) = pending.lock().await.remove(&id) else {
        debug!(id, "acp response for unknown request");
        return true;
    };
    if let Some(c) = prompt_cleanup {
        c.clear_if_prompt_response(id).await;
    }
    if let Some(err) = msg.get("error") {
        let _ = tx.send(Err(super::transport::format_jsonrpc_error(err)));
    } else if let Some(res) = msg.get("result") {
        let _ = tx.send(Ok(res.clone()));
    } else {
        let _ = tx.send(Err("acp response missing result/error".into()));
    }
    true
}

pub(crate) async fn handle_incoming_line(
    line: &str,
    pending: &Arc<Mutex<HashMap<u64, ResponseTx>>>,
    stdin: &Arc<Mutex<ChildStdin>>,
    prompt_cleanup: Option<&PromptRpcCleanup>,
    acp_verbose: bool,
) {
    let msg: Value = match serde_json::from_str(line) {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, "acp stdout JSON parse error");
            return;
        }
    };
    match msg.get("method").and_then(|m| m.as_str()) {
        None => {
            let _ = dispatch_response(&msg, pending, prompt_cleanup).await;
        }
        Some("session/update") => {
            trace!(target: "malvin::acp", update = %msg, "session/update");
        }
        Some("session/request_permission") => {
            // Auto-`allow-always` keeps the headless daemon from blocking on tool prompts; it also means
            // compromised model output or malicious prompt content can drive tool execution—an explicit
            // product/security tradeoff (see `.llm_style/george.md`).
            let Some(id) = request_permission_correlation_id(&msg) else {
                warn!(
                    target: "malvin::acp",
                    "session/request_permission missing correlation id (top-level or params.id/requestId); cannot reply"
                );
                return;
            };
            let body = json!({
                "jsonrpc": "2.0",
                "id": id.clone(),
                "result": {
                    "outcome": { "outcome": "selected", "optionId": "allow-always" }
                }
            });
            let line = match serde_json::to_string(&body) {
                Ok(l) => l,
                Err(e) => {
                    error!(error = %e, "failed to answer session/request_permission");
                    return;
                }
            };
            if let Err(e) = write_rpc_line(stdin, &line, acp_verbose).await {
                error!(error = %e, "failed to answer session/request_permission");
            }
        }
        Some(method) => {
            trace!(target: "malvin::acp", method, "acp notification or server request (ignored)");
        }
    }
}

pub(crate) async fn reader_loop(
    stdout: ChildStdout,
    pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    stdin: Arc<Mutex<ChildStdin>>,
    reader_dead: Arc<AtomicBool>,
    trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    prompt_cleanup: Option<Arc<PromptRpcCleanup>>,
    acp_verbose: bool,
) {
    let mut lines = BufReader::new(stdout).lines();
    let mut verbose_coalesce = VerboseIoCoalescer::default();
    let mut trace_coalesce = TraceChunkCoalescer::default();
    while let Ok(Some(line)) = lines.next_line().await {
        reader_loop_verbose_and_trace_line(
            &line,
            acp_verbose,
            &trace_writer,
            &mut verbose_coalesce,
            &mut trace_coalesce,
        )
        .await;
        let pc = prompt_cleanup.as_deref();
        handle_incoming_line(&line, &pending, &stdin, pc, acp_verbose).await;
    }
    if acp_verbose {
        verbose_coalesce.flush_all();
    }
    {
        let mut g = trace_writer.lock().await;
        if let Some(ref mut f) = *g {
            for tl in trace_coalesce.flush_all() {
                trace_file_write_line(f, &tl).await;
            }
        }
    }
    reader_dead.store(true, Ordering::SeqCst);
    let mut g = pending.lock().await;
    for (_, tx) in g.drain() {
        let _ = tx.send(Err("acp stdout closed".into()));
    }
}

pub(crate) fn spawn_acp_stdout_reader(
    stdout: ChildStdout,
    pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    stdin: Arc<Mutex<ChildStdin>>,
    reader_dead: Arc<AtomicBool>,
    trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    prompt_cleanup: Arc<PromptRpcCleanup>,
    acp_verbose: bool,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        reader_loop(
            stdout,
            pending,
            stdin,
            reader_dead,
            trace_writer,
            Some(prompt_cleanup),
            acp_verbose,
        )
        .await;
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use tokio::process::Command;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_dispatch_response_ok_error_orphans_and_malformed() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));

        let (tx, rx) = oneshot::channel();
        pending.lock().await.insert(1, tx);
        let ok = json!({"jsonrpc": "2.0", "id": 1, "result": {"a": 1}});
        assert!(dispatch_response(&ok, &pending, None).await);
        assert_eq!(rx.await.unwrap().unwrap()["a"], 1);

        let (tx2, rx2) = oneshot::channel();
        pending.lock().await.insert(2, tx2);
        let err = json!({"jsonrpc": "2.0", "id": 2, "error": {"message": "e"}});
        assert!(dispatch_response(&err, &pending, None).await);
        assert!(rx2.await.unwrap().unwrap_err().contains("message"));

        let (tx3, rx3) = oneshot::channel();
        pending.lock().await.insert(3, tx3);
        let neither = json!({"jsonrpc": "2.0", "id": 3});
        assert!(dispatch_response(&neither, &pending, None).await);
        assert!(
            rx3.await
                .unwrap()
                .unwrap_err()
                .contains("missing result/error")
        );

        let no_id = json!({"jsonrpc": "2.0", "result": {}});
        assert!(!dispatch_response(&no_id, &pending, None).await);

        let bad_id = json!({"jsonrpc": "2.0", "id": "x", "result": {}});
        assert!(!dispatch_response(&bad_id, &pending, None).await);

        let orphan = json!({"jsonrpc": "2.0", "id": 99, "result": {}});
        assert!(dispatch_response(&orphan, &pending, None).await);
    }

    /// JSON-RPC 2.0 allows `id` to be a JSON number; serde may represent small integers as `i64`.
    #[tokio::test]
    async fn dispatch_resolves_pending_when_response_id_is_i64() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = oneshot::channel();
        pending.lock().await.insert(7, tx);
        let msg = json!({"jsonrpc": "2.0", "id": 7i64, "result": {"v": 1}});
        assert!(dispatch_response(&msg, &pending, None).await);
        assert_eq!(rx.await.unwrap().unwrap()["v"], 1);
    }

    /// JSON-RPC 2.0 allows `id` to be a string. Peers may echo a numeric request id as a string in the response.
    #[tokio::test]
    async fn dispatch_resolves_pending_when_response_id_is_decimal_string() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = oneshot::channel();
        pending.lock().await.insert(1, tx);
        let msg = json!({"jsonrpc": "2.0", "id": "1", "result": {"v": 42}});
        assert!(
            dispatch_response(&msg, &pending, None).await,
            "string id should match pending request 1"
        );
        assert_eq!(rx.await.unwrap().unwrap()["v"], 42);
    }

    #[tokio::test]
    async fn test_handle_incoming_line_parse_error_and_extension_method() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut child = Command::new("sleep")
            .arg("30")
            .stdin(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let _reap = tokio::spawn(async move {
            let _ = child.kill().await;
            let _ = child.wait().await;
        });

        handle_incoming_line("%%%", &pending, &stdin, None, false).await;
        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"cursor/task","params":{}}"#,
            &pending,
            &stdin,
            None,
            false,
        )
        .await;
    }

    #[test]
    fn session_update_chunk_parts_message() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"x","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"want to work "}}}}"#;
        let v: Value = serde_json::from_str(line).unwrap();
        let (k, t) = session_update_chunk_parts(&v).expect("chunk");
        assert!(matches!(k, super::SessionUpdateChunkKind::Message));
        assert_eq!(t, "want to work ");
    }

    #[test]
    fn session_update_chunk_parts_thought() {
        let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_thought_chunk","content":{"type":"text","text":"thinking"}}}}"#;
        let v: Value = serde_json::from_str(line).unwrap();
        let (k, t) = session_update_chunk_parts(&v).expect("chunk");
        assert!(matches!(k, super::SessionUpdateChunkKind::Thought));
        assert_eq!(t, "thinking");
    }

    #[test]
    fn session_update_chunk_parts_skips_non_session_update() {
        let v: Value = serde_json::from_str(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#).unwrap();
        assert!(session_update_chunk_parts(&v).is_none());
    }

    #[test]
    fn coalesce_append_emits_at_newline_without_newline_in_output() {
        let mut buf = String::new();
        let mut out = Vec::new();
        coalesce_append_chunk(&mut buf, "hello\nworld", &mut out);
        assert_eq!(out, vec!["hello".to_string()]);
        assert_eq!(buf, "world");
        coalesce_append_chunk(&mut buf, "\n", &mut out);
        assert_eq!(out, vec!["hello".to_string(), "world".to_string()]);
        assert!(buf.is_empty());
    }

    #[test]
    fn coalesce_append_emits_at_cap_then_carries_rest() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        let mut buf = String::new();
        let mut out = Vec::new();
        let prefix: String = (0..max).map(|_| 'x').collect();
        let extra = format!("{prefix}abcde");
        coalesce_append_chunk(&mut buf, &extra, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].chars().count(), max);
        assert_eq!(buf, "abcde");
    }

    #[test]
    fn coalesce_append_multiple_cap_rounds_without_newline() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        let mut buf = String::new();
        let mut out = Vec::new();
        let n = max * 2 + 40;
        coalesce_append_chunk(&mut buf, &"x".repeat(n), &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].len(), max);
        assert_eq!(out[1].len(), max);
        assert_eq!(buf.len(), 40);
    }

    #[test]
    fn coalesce_append_cap_then_remainder_flushed_at_newline() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        let mut buf = String::new();
        let mut out = Vec::new();
        let chunk = format!("{}\n", "a".repeat(max + 5));
        coalesce_append_chunk(&mut buf, &chunk, &mut out);
        assert_eq!(out, vec!["a".repeat(max), "aaaaa".to_string()]);
        assert!(buf.is_empty());
    }

    #[test]
    fn coalesce_append_only_newlines_emits_nothing() {
        let mut buf = String::new();
        let mut out = Vec::new();
        coalesce_append_chunk(&mut buf, "\n\n\n", &mut out);
        assert!(out.is_empty());
        assert!(buf.is_empty());
    }

    #[test]
    fn coalesce_char_boundary_at_past_end_yields_len() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        assert_eq!(coalesce_char_boundary_at("hi", 99), 2);
        assert_eq!(coalesce_char_boundary_at("", 1), 0);
        let xs = "x".repeat(max);
        assert_eq!(coalesce_char_boundary_at(&xs, max), xs.len());
    }

    #[test]
    fn coalesce_flush_cap_drains_exactly_cap_char_buffer() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        let mut buf = "x".repeat(max);
        let mut out = Vec::new();
        coalesce_flush_cap(&mut buf, &mut out);
        assert_eq!(out, vec!["x".repeat(max)]);
        assert!(buf.is_empty());
    }

    #[test]
    fn coalesce_flush_cap_multiple_iterations() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        let mut buf = "y".repeat(max * 3 + 10);
        let mut out = Vec::new();
        coalesce_flush_cap(&mut buf, &mut out);
        assert_eq!(out.len(), 3);
        assert_eq!(buf.len(), 10);
    }

    #[test]
    fn coalesce_flush_nonempty_direct() {
        let mut buf = String::from("hello");
        let mut out = Vec::new();
        coalesce_flush_nonempty(&mut buf, &mut out);
        assert_eq!(out, vec!["hello".to_string()]);
        assert!(buf.is_empty());
        coalesce_flush_nonempty(&mut buf, &mut out);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn coalesce_append_splits_on_unicode_scalar_count() {
        let max = super::ACP_VERBOSE_COALESCE_MAX;
        let mut buf = String::new();
        let mut out = Vec::new();
        let s = "€".repeat(max + 5);
        coalesce_append_chunk(&mut buf, &s, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].chars().count(), max);
        assert_eq!(buf.chars().count(), 5);
    }

    #[test]
    fn verbose_io_coalescer_feed_and_flush_all_covers_paths() {
        let mut c = VerboseIoCoalescer::default();
        c.feed(SessionUpdateChunkKind::Message, "hello");
        c.feed(SessionUpdateChunkKind::Thought, "think");
        c.flush_all();
    }

    #[test]
    fn trace_chunk_coalescer_merges_two_small_message_chunks() {
        let v = json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"x"}}}});
        let mut c = TraceChunkCoalescer::default();
        assert!(c
            .feed(SessionUpdateChunkKind::Message, "hel", &v)
            .is_empty());
        assert!(c
            .feed(SessionUpdateChunkKind::Message, "lo", &v)
            .is_empty());
        let fin = c.flush_all();
        assert_eq!(fin.len(), 1);
        let parsed: Value = serde_json::from_str(&fin[0]).unwrap();
        assert_eq!(
            parsed
                .pointer("/params/update/content/text")
                .and_then(Value::as_str),
            Some("hello")
        );
    }

    #[test]
    fn patch_session_update_chunk_text_requires_content_path() {
        assert!(super::patch_session_update_chunk_text(json!({"a": 1}), "x").is_none());
    }

    #[tokio::test]
    async fn write_trace_line_coalesced_writes_raw_when_not_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("coalesce-trace.jsonl");
        let mut f = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .await
            .unwrap();
        let mut c = TraceChunkCoalescer::default();
        let raw = "not-json-line";
        super::write_trace_line_coalesced(raw, &mut f, &mut c, &None).await;
        drop(f);
        let s = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(s.trim_end(), raw);
    }

    #[test]
    fn trace_chunk_coalescer_emits_at_cap_like_verbose() {
        let max = ACP_VERBOSE_COALESCE_MAX;
        let v = json!({"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"."}}}});
        let mut c = TraceChunkCoalescer::default();
        let chunk = "x".repeat(max + 10);
        let out = c.feed(SessionUpdateChunkKind::Message, &chunk, &v);
        assert_eq!(out.len(), 1);
        let p0: Value = serde_json::from_str(&out[0]).unwrap();
        assert_eq!(
            p0.pointer("/params/update/content/text")
                .and_then(Value::as_str)
                .map(|s| s.chars().count()),
            Some(max)
        );
        let fin = c.flush_all();
        assert_eq!(fin.len(), 1);
        let p1: Value = serde_json::from_str(&fin[0]).unwrap();
        assert_eq!(
            p1.pointer("/params/update/content/text")
                .and_then(Value::as_str)
                .map(|s| s.len()),
            Some(10)
        );
    }

    #[test]
    fn jsonrpc_response_id_parses_u64_and_decimal_string_and_rejects_garbage() {
        assert_eq!(jsonrpc_response_id_as_u64(&json!(42u64)), Some(42));
        assert_eq!(jsonrpc_response_id_as_u64(&json!(42i64)), Some(42));
        assert_eq!(jsonrpc_response_id_as_u64(&json!("99")), Some(99));
        assert_eq!(jsonrpc_response_id_as_u64(&json!("not-a-number")), None);
        assert_eq!(jsonrpc_response_id_as_u64(&json!(-1i64)), None);
        assert_eq!(jsonrpc_response_id_as_u64(&json!(null)), None);
    }

    #[test]
    fn request_permission_correlation_id_top_level_params_and_request_id() {
        let top = json!({"jsonrpc":"2.0","id":1,"params":{"id":2}});
        assert_eq!(request_permission_correlation_id(&top), top.get("id"));
        let nested = json!({"jsonrpc":"2.0","method":"session/request_permission","params":{"id":2}});
        assert_eq!(request_permission_correlation_id(&nested), nested.pointer("/params/id"));
        let req_id = json!({"params":{"requestId":"9"}});
        assert_eq!(request_permission_correlation_id(&req_id), req_id.pointer("/params/requestId"));
        let none = json!({"method":"session/request_permission","params":{}});
        assert_eq!(request_permission_correlation_id(&none), None);
    }

    #[test]
    fn test_permission_reply_shape() {
        let id = json!(42u64);
        let body = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "outcome": { "outcome": "selected", "optionId": "allow-always" }
            }
        });
        assert!(body.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_session_update_and_permission_replies() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut child = Command::new("sleep")
            .arg("5")
            .stdin(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"session/update","params":{"t":1}}"#,
            &pending,
            &stdin,
            None,
            false,
        )
        .await;

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","id":42,"method":"session/request_permission","params":{}}"#,
            &pending,
            &stdin,
            None,
            false,
        )
        .await;

        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    /// KPOP: `session/request_permission` with no correlation id anywhere still skips `write_rpc_line`.
    #[cfg(unix)]
    #[tokio::test]
    async fn kpop_permission_without_correlation_id_writes_nothing_to_child_stdin() {
        use tokio::io::AsyncReadExt;

        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("cat");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let mut stdout = child.stdout.take().expect("stdout");

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{}}"#,
            &pending,
            &stdin,
            None,
            false,
        )
        .await;

        drop(stdin);
        let mut received = Vec::new();
        stdout
            .read_to_end(&mut received)
            .await
            .expect("read stdout");
        let _ = child.wait().await.expect("wait cat");
        assert!(
            received.is_empty(),
            "expected no bytes written for permission message without id; got {:?}",
            String::from_utf8_lossy(&received)
        );
    }

    /// Permission prompt with `id` only under `params` must still get an allow-always JSON-RPC reply line.
    #[cfg(unix)]
    #[tokio::test]
    async fn permission_with_id_in_params_writes_allow_always_reply_line() {
        use tokio::io::AsyncReadExt;

        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut child = Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("cat");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let mut stdout = child.stdout.take().expect("stdout");

        handle_incoming_line(
            r#"{"jsonrpc":"2.0","method":"session/request_permission","params":{"id":77}}"#,
            &pending,
            &stdin,
            None,
            false,
        )
        .await;

        drop(stdin);
        let mut received = Vec::new();
        stdout
            .read_to_end(&mut received)
            .await
            .expect("read stdout");
        let _ = child.wait().await.expect("wait cat");
        let line = String::from_utf8_lossy(&received);
        assert!(
            line.contains("allow-always") && (line.contains(r#""id":77"#) || line.contains(r#""id": 77"#)),
            "expected allow-always reply echoing id 77; got {line:?}"
        );
    }

    #[tokio::test]
    async fn test_permission_json_or_write_failure_is_logged() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut child = Command::new("true")
            .stdin(Stdio::piped())
            .spawn()
            .expect("true");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let _ = child.wait().await;
        handle_incoming_line(
            r#"{"jsonrpc":"2.0","id":9,"method":"session/request_permission","params":{}}"#,
            &pending,
            &stdin,
            None,
            false,
        )
        .await;
    }

    #[tokio::test]
    async fn test_reader_loop_drains_pending_on_stdout_eof() {
        let mut child = Command::new("true")
            .stdout(Stdio::piped())
            .spawn()
            .expect("true");
        let stdout = child.stdout.take().expect("stdout");
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = oneshot::channel();
        pending.lock().await.insert(7, tx);
        let mut stdin_holder = Command::new("sleep")
            .arg("2")
            .stdin(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(stdin_holder.stdin.take().expect("stdin")));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let trace_writer: Arc<Mutex<Option<tokio::fs::File>>> = Arc::new(Mutex::new(None));
        let busy = Arc::new(AtomicBool::new(false));
        let prompt_rpc_id = Arc::new(AtomicU64::new(0));
        let prompt_cleanup = Arc::new(PromptRpcCleanup {
            busy,
            trace_writer: trace_writer.clone(),
            prompt_rpc_id,
            idle_notify: None,
        });
        let waiter = spawn_acp_stdout_reader(
            stdout,
            pending.clone(),
            stdin,
            reader_dead,
            trace_writer,
            prompt_cleanup,
            false,
        );
        let err = rx.await.unwrap().unwrap_err();
        assert!(err.contains("closed") || err.contains("acp"));
        waiter.await.unwrap();
        let _ = child.wait().await;
        let _ = stdin_holder.kill().await;
    }

    #[tokio::test]
    async fn dispatch_clears_prompt_cleanup_when_id_matches() {
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let busy = Arc::new(AtomicBool::new(true));
        let trace_writer: Arc<Mutex<Option<tokio::fs::File>>> = Arc::new(Mutex::new(None));
        let prompt_rpc_id = Arc::new(AtomicU64::new(5));
        let cleanup = PromptRpcCleanup {
            busy: busy.clone(),
            trace_writer: trace_writer.clone(),
            prompt_rpc_id: prompt_rpc_id.clone(),
            idle_notify: None,
        };
        let (tx, rx) = oneshot::channel();
        pending.lock().await.insert(5, tx);
        let msg = json!({"jsonrpc": "2.0", "id": 5, "result": {"stopReason": "end"}});
        assert!(dispatch_response(&msg, &pending, Some(&cleanup)).await);
        assert!(rx.await.unwrap().unwrap()["stopReason"] == "end");
        assert!(!busy.load(Ordering::SeqCst));
        assert_eq!(prompt_rpc_id.load(Ordering::SeqCst), 0);
        assert!(trace_writer.lock().await.is_none());
    }
}
