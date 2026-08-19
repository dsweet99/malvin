use std::path::Path;

use serde_json::Value;

use super::env::HerdrEnv;
use super::identity::{display_title, herdr_live_name};
use super::request::{
    clear_agent_authority, next_seq, rename_agent, report_agent, report_agent_session,
    report_metadata_sparse,
};
use super::send::send_request_checked;
use super::trace::log_herdr_failure;

pub fn emit_bind_reports(env: &HerdrEnv, session_id: Option<&str>, run_dir: Option<&Path>) {
    let sock = env.socket_path.as_path();
    for (phase, req) in bind_requests(env.pane_id.as_str(), session_id, run_dir) {
        verified(sock, run_dir, phase, &req);
    }
}

fn bind_requests(
    pane: &str,
    session_id: Option<&str>,
    run_dir: Option<&Path>,
) -> Vec<(&'static str, Value)> {
    let path = run_dir.and_then(abs_path_string);
    let title = display_title();
    let mut out = vec![
        ("clear", clear_agent_authority(pane, next_seq())),
        (
            "session",
            report_agent_session(pane, session_id, path.as_deref(), next_seq()),
        ),
        (
            "working",
            report_agent(pane, "working", session_id, next_seq()),
        ),
        (
            "metadata",
            report_metadata_sparse(pane, title.as_deref(), next_seq()),
        ),
    ];
    if let Some(sid) = session_id {
        out.push(("rename", rename_agent(pane, &herdr_live_name(sid))));
    }
    out
}

fn verified(sock: &Path, run_dir: Option<&Path>, phase: &str, request: &Value) {
    if let Err(detail) = send_request_checked(sock, request) {
        log_herdr_failure(run_dir, phase, &detail);
    }
}

fn abs_path_string(path: &Path) -> Option<String> {
    path.canonicalize()
        .ok()
        .or_else(|| Some(path.to_path_buf()))
        .map(|p| p.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::{abs_path_string, bind_requests, emit_bind_reports};
    use std::path::PathBuf;

    #[test]
    fn abs_path_string_falls_back_when_missing() {
        let p = PathBuf::from("/no/such/malvin_herdr_path_xyz");
        let s = abs_path_string(&p).expect("fallback");
        assert!(s.contains("malvin_herdr_path_xyz"));
        let steps = bind_requests("pane", Some("20260804_140533_4gk60f1m"), None);
        assert!(steps.iter().any(|(p, _)| *p == "rename"));
        let _ = emit_bind_reports;
    }
}
