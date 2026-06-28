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
