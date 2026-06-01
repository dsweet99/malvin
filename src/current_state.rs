//! Renders the `{{ current_state }}` prompt placeholder for agent context.

use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::sandbox_oom::gate_iteration_oom_killed;
use crate::mem_limit_config::{
    format_memory_gib, load_mem_limit_bytes, system_total_memory_bytes,
};

/// Builds the multi-line `current_state` body for prompt templates.
#[must_use]
pub fn format_current_state(
    work_dir: &Path,
    gate_iteration: Option<usize>,
    artifacts: Option<&RunArtifacts>,
) -> String {
    [
        format!("User: {}", format_user_identity()),
        format!("Date/time: {}", format_local_datetime()),
        format_sandbox_memory_line(work_dir),
        format_retry_line(gate_iteration, artifacts),
    ]
    .join("\n")
}

#[must_use]
pub fn format_user_identity() -> String {
    let name = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    if let Some(uid) = effective_user_id() {
        format!("{name} (uid {uid})")
    } else {
        name
    }
}

#[cfg(unix)]
fn effective_user_id() -> Option<u32> {
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
}

#[cfg(not(unix))]
fn effective_user_id() -> Option<u32> {
    None
}

#[must_use]
pub fn format_local_datetime() -> String {
    chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S %Z")
        .to_string()
}

#[must_use]
pub fn format_sandbox_memory_line(work_dir: &Path) -> String {
    let limit = load_mem_limit_bytes(work_dir);
    let rss = current_sandbox_rss_bytes().unwrap_or(0);
    let available = limit.saturating_sub(rss);
    let mut parts = vec![
        format!("limit {}", format_memory_gib(limit)),
        format!("in use {}", format_memory_gib(rss)),
        format!("available {}", format_memory_gib(available)),
    ];
    if let Some(host) = system_total_memory_bytes() {
        parts.push(format!("host total {}", format_memory_gib(host)));
    }
    format!("Sandbox memory: {}", parts.join(", "))
}

fn current_sandbox_rss_bytes() -> Option<u64> {
    #[cfg(unix)]
    {
        let stack = crate::active_agent_heartbeat::active_agent_process_group_for_stats()?;
        crate::malvin_sandbox::malvin_session_rss_bytes(Some(stack.pgid), &stack.spawn_baseline)
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[must_use]
pub fn format_retry_line(gate_iteration: Option<usize>, artifacts: Option<&RunArtifacts>) -> String {
    let Some(iter) = gate_iteration.filter(|&i| i > 0) else {
        return "Retry: not a retry (first session in this malvin run).".to_string();
    };
    if iter == 1 {
        return "Retry: not a retry (first outer gate-loop session).".to_string();
    }
    let retry_num = iter - 1;
    let reasons = infer_gate_retry_reasons(artifacts, iter);
    if reasons.is_empty() {
        format!(
            "Retry: yes — outer gate-loop session {iter} (retry #{retry_num}); reason not recorded."
        )
    } else {
        format!(
            "Retry: yes — outer gate-loop session {iter} (retry #{retry_num}); reason: {}.",
            reasons.join("; ")
        )
    }
}

fn infer_gate_retry_reasons(artifacts: Option<&RunArtifacts>, iteration: usize) -> Vec<String> {
    let Some(artifacts) = artifacts else {
        return Vec::new();
    };
    let prev = iteration.saturating_sub(1);
    if prev == 0 {
        return Vec::new();
    }
    let mut reasons = Vec::new();
    append_unsolved_reason(&mut reasons, artifacts, prev);
    append_oom_reason(&mut reasons, artifacts, prev);
    append_gates_reason(&mut reasons, artifacts, prev);
    reasons
}

fn append_unsolved_reason(reasons: &mut Vec<String>, artifacts: &RunArtifacts, prev: usize) {
    if read_prev_exp_solved(artifacts, prev) == Some(false) {
        reasons.push(
            "previous KPop session did not write ## KPOP_SOLVED to its experiment log".to_string(),
        );
    }
}

fn append_oom_reason(reasons: &mut Vec<String>, artifacts: &RunArtifacts, prev: usize) {
    if gate_iteration_oom_killed(artifacts, prev) {
        reasons.push("previous agent killed: sandbox exceeded memory limit (OOM)".to_string());
    }
}

fn append_gates_reason(reasons: &mut Vec<String>, artifacts: &RunArtifacts, prev: usize) {
    if read_prev_exp_solved(artifacts, prev) == Some(true) && reasons.is_empty() {
        reasons.push(
            "quality gates did not pass after previous ## KPOP_SOLVED session".to_string(),
        );
    }
}

fn read_prev_exp_solved(artifacts: &RunArtifacts, prev: usize) -> Option<bool> {
    let path = artifacts.gate_exp_log_path(prev);
    if !path.is_file() {
        return None;
    }
    std::fs::read_to_string(path)
        .ok()
        .map(|text| crate::kpop_progression::agent_declared_success(&text))
}

#[cfg(test)]
#[path = "current_state_tests.rs"]
mod current_state_tests;
