use std::path::Path;

use crate::artifacts::RunArtifacts;
use crate::mem_limit_config::{format_memory_gib, load_mem_limit_bytes, system_total_memory_bytes};
use crate::sandbox_oom::gate_iteration_oom_killed;

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
    let login = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let uid = effective_user_id();
    let full_name = uid.and_then(passwd_gecos_full_name);
    assemble_user_identity(&login, uid, full_name.as_deref())
}

#[must_use]
pub fn assemble_user_identity(login: &str, uid: Option<u32>, full_name: Option<&str>) -> String {
    let include_full_name = full_name.filter(|name| !name.is_empty() && *name != login);
    match (uid, include_full_name) {
        (Some(uid), Some(name)) => format!("{login} (uid {uid}, {name})"),
        (Some(uid), None) => format!("{login} (uid {uid})"),
        (None, _) => login.to_string(),
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod passwd_gecos {
    #![allow(unsafe_code)]

    #[must_use]
    pub fn full_name(uid: u32) -> Option<String> {
        let pw = unsafe { libc::getpwuid(uid) };
        if pw.is_null() {
            return None;
        }
        let gecos = unsafe { std::ffi::CStr::from_ptr((*pw).pw_gecos) };
        let full_name = gecos
            .to_str()
            .ok()?
            .split(',')
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())?;
        Some(full_name.to_string())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn passwd_gecos_full_name(uid: u32) -> Option<String> {
    passwd_gecos::full_name(uid)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn passwd_gecos_full_name(_uid: u32) -> Option<String> {
    None
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
pub fn format_retry_line(
    gate_iteration: Option<usize>,
    artifacts: Option<&RunArtifacts>,
) -> String {
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
    append_oom_reason(&mut reasons, artifacts, prev);
    append_gates_reason(&mut reasons, artifacts, prev);
    append_unsolved_reason(&mut reasons, artifacts, prev);
    reasons
}

fn append_unsolved_reason(reasons: &mut Vec<String>, artifacts: &RunArtifacts, prev: usize) {
    if prev_exp_log_ran(artifacts, prev) && reasons.is_empty() {
        reasons.push(
            "requirements still unsatisfied after previous router session".to_string(),
        );
    }
}

fn prev_exp_log_ran(artifacts: &RunArtifacts, prev: usize) -> bool {
    artifacts.gate_exp_log_path(prev).is_file()
}

fn append_oom_reason(reasons: &mut Vec<String>, artifacts: &RunArtifacts, prev: usize) {
    if gate_iteration_oom_killed(artifacts, prev) {
        reasons.push("previous agent killed: sandbox exceeded memory limit (OOM)".to_string());
    }
}

fn append_gates_reason(reasons: &mut Vec<String>, artifacts: &RunArtifacts, _prev: usize) {
    if previous_quality_gates_failed(artifacts) {
        reasons.push("quality gates did not pass after previous router session".to_string());
    }
}

fn previous_quality_gates_failed(artifacts: &RunArtifacts) -> bool {
    review_marks_checks_failed(artifacts) || quality_gates_log_has_nonzero_exit(artifacts)
}

fn review_marks_checks_failed(artifacts: &RunArtifacts) -> bool {
    std::fs::read_to_string(artifacts.artifact_review_md())
        .is_ok_and(|text| text.contains("Checks do not pass"))
}

fn quality_gates_log_has_nonzero_exit(artifacts: &RunArtifacts) -> bool {
    let Ok(log) = std::fs::read_to_string(artifacts.quality_gates_log_path()) else {
        return false;
    };
    log.lines().any(|line| {
        line.strip_prefix("exit code: ")
            .is_some_and(|code| code.trim() != "0")
    })
}

#[cfg(test)]
#[path = "current_state_tests.rs"]
mod current_state_tests;

#[cfg(test)]
#[path = "current_state_tests_tail.rs"]
mod current_state_tests_tail;
