//! ACP-shaped trace emission for `--mini` runs.

use std::sync::atomic::{AtomicU64, Ordering};

use malvin_mini::ResponseUsage;
use serde_json::{json, Value};

use crate::acp::AcpJsonlTrace;

static MINI_TOOL_CALL_SEQ: AtomicU64 = AtomicU64::new(0);

fn next_tool_call_id() -> String {
    let n = MINI_TOOL_CALL_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("mini_tool_{n}")
}

fn session_update_message(update: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": { "update": update }
    })
}

pub(crate) fn mini_trace_name() -> String {
    "mini".to_string()
}

pub(crate) fn trace_for_run_dir(run_dir: &std::path::Path) -> AcpJsonlTrace {
    AcpJsonlTrace::new(run_dir.join("trace.jsonl"), mini_trace_name())
}

pub(crate) fn append_out_raw(trace: &AcpJsonlTrace, text: &str) {
    trace.append_line("out", text);
}

pub(crate) fn append_in_json(trace: &AcpJsonlTrace, message: &Value) {
    trace.append_line("in", &message.to_string());
}

pub(crate) fn emit_agent_message_chunk(trace: &AcpJsonlTrace, text: &str) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": text }
    }));
    append_in_json(trace, &msg);
}

pub(crate) fn emit_agent_thought_chunk(trace: &AcpJsonlTrace, text: &str) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_thought_chunk",
        "content": { "type": "text", "text": text }
    }));
    append_in_json(trace, &msg);
}

pub(crate) fn emit_llm_usage(trace: &AcpJsonlTrace, usage: &ResponseUsage) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": "" },
        "miniUsage": {
            "prompt_tokens": usage.prompt_tokens,
            "completion_tokens": usage.completion_tokens,
            "total_tokens": usage.total_tokens,
            "cost": usage.cost,
        }
    }));
    append_in_json(trace, &msg);
}

pub(crate) fn emit_bash_tool_call(
    trace: &AcpJsonlTrace,
    kind: &str,
    command: &str,
    exit_code: i32,
) {
    let id = next_tool_call_id();
    let pending = session_update_message(json!({
        "sessionUpdate": "tool_call",
        "toolCallId": id,
        "kind": kind,
        "status": "pending",
        "title": kind,
        "rawInput": { "command": command }
    }));
    append_in_json(trace, &pending);
    let done = session_update_message(json!({
        "sessionUpdate": "tool_call_update",
        "toolCallId": id,
        "kind": kind,
        "status": "completed",
        "rawOutput": { "exitCode": exit_code }
    }));
    append_in_json(trace, &done);
}

pub(crate) fn emit_mini_terminal(trace: &AcpJsonlTrace, record: &super::terminal::MiniTerminalRecord) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": "" },
        "miniTerminal": {
            "reason": record.reason.as_str(),
            "http_turn_count": record.http_turn_count,
            "bash_exec_count": record.bash_exec_count,
            "phase_at_exit": record.phase_at_exit.as_str(),
        }
    }));
    append_in_json(trace, &msg);
}

pub(crate) struct MiniPromptShrinkTrace<'a> {
    pub attempt: u32,
    pub messages_before: usize,
    pub messages_after: usize,
    pub bytes_removed: usize,
    pub strategy: &'a str,
}

pub(crate) fn emit_mini_prompt_shrink(trace: &AcpJsonlTrace, shrink: MiniPromptShrinkTrace<'_>) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": "" },
        "miniPromptShrink": {
            "attempt": shrink.attempt,
            "messages_before": shrink.messages_before,
            "messages_after": shrink.messages_after,
            "bytes_removed": shrink.bytes_removed,
            "strategy": shrink.strategy,
        }
    }));
    append_in_json(trace, &msg);
}

pub(crate) fn emit_mini_prompt_shrink_stalled(trace: &AcpJsonlTrace) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": "" },
        "miniPromptShrinkStalled": true
    }));
    append_in_json(trace, &msg);
}

pub(crate) fn emit_mini_retry_fork(trace: &AcpJsonlTrace, ledger: &super::retry_fork::RetryForkLedger) {
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": "" },
        "miniRetryFork": {
            "prompt_index": ledger.prompt_index,
            "attempt": ledger.attempt,
            "message_checkpoint_len": ledger.message_checkpoint_len,
            "workspace_manifest_hash": ledger.workspace_manifest_hash,
            "bash_commands": ledger.bash_commands,
            "outcome": ledger.outcome.as_str(),
            "strategy": ledger.strategy.as_str(),
        }
    }));
    append_in_json(trace, &msg);
}

/// Raw `OpenRouter` HTTP bodies in trace are capped at 64 KiB to avoid bloating run dirs.
const HTTP_EXCHANGE_BODY_TRACE_CAP: usize = 64 * 1024;

fn truncate_http_body_for_trace(body: &str) -> String {
    if body.len() <= HTTP_EXCHANGE_BODY_TRACE_CAP {
        body.to_string()
    } else {
        format!(
            "{}…[truncated {} bytes]",
            &body[..HTTP_EXCHANGE_BODY_TRACE_CAP],
            body.len() - HTTP_EXCHANGE_BODY_TRACE_CAP
        )
    }
}

pub struct MiniHttpExchangeRecord<'a> {
    pub attempt: u32,
    pub status: Option<u16>,
    pub body: Option<&'a str>,
    pub error: Option<String>,
}

pub(crate) fn emit_mini_http_exchange(trace: &AcpJsonlTrace, record: MiniHttpExchangeRecord<'_>) {
    let body_value = record.body.map(truncate_http_body_for_trace);
    let msg = session_update_message(json!({
        "sessionUpdate": "agent_message_chunk",
        "content": { "type": "text", "text": "" },
        "miniHttpExchange": {
            "attempt": record.attempt,
            "status": record.status,
            "body": body_value,
            "error": record.error,
        }
    }));
    append_in_json(trace, &msg);
}

#[cfg(test)]
mod acp_trace_shim_tests {
    use super::{emit_mini_http_exchange, trace_for_run_dir, truncate_http_body_for_trace, MiniHttpExchangeRecord, HTTP_EXCHANGE_BODY_TRACE_CAP};

    #[test]
    fn truncate_http_body_for_trace_at_cap_is_unchanged() {
        let body = "a".repeat(HTTP_EXCHANGE_BODY_TRACE_CAP);
        assert_eq!(truncate_http_body_for_trace(&body), body);
    }

    #[test]
    fn emit_mini_http_exchange_accepts_all_none_fields() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let trace = trace_for_run_dir(tmp.path());
        emit_mini_http_exchange(&trace, MiniHttpExchangeRecord {
            attempt: 0,
            status: None,
            body: None,
            error: None,
        });
        let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(text.contains("\"body\":null"));
    }
}
