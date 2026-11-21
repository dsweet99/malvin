use std::path::Path;

use serde_json::{Value, json};

use super::env::HerdrEnv;
use super::identity::{display_title, herdr_live_name};
use super::request::{
    clear_agent_authority, next_request_id, next_seq, rename_agent, report_agent,
    report_agent_session, report_metadata_sparse,
};
use super::send::{send_request_reply, send_request_with_retry};
use super::trace::log_herdr_failure;

pub fn emit_bind_reports(env: &HerdrEnv, session_id: Option<&str>, run_dir: Option<&Path>) {
    for (phase, req) in bind_requests(env.pane_id.as_str(), session_id, run_dir) {
        if let Err(detail) = send_request_with_retry(&env.socket_path, &req) {
            log_herdr_failure(run_dir, phase, &detail);
            if phase == "rename" && detail.starts_with("agent_not_found:") {
                explain_missing_agent(env, run_dir);
            }
        }
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

fn explain_missing_agent(env: &HerdrEnv, run_dir: Option<&Path>) {
    let pane = &env.pane_id;
    let query = json!({"id": next_request_id(), "method": "pane.get", "params": {"pane_id": pane}});
    let Ok(result) = send_request_reply(&env.socket_path, &query) else {
        return;
    };
    if let Some(owner) = conflicting_session(&result["pane"]) {
        let detail = format!(
            "Herdr pane {pane} still holds a saved {owner} session from an earlier run, so Herdr ignores malvin's working status in this pane. Run malvin in a new Herdr pane to see its status."
        );
        log_herdr_failure(run_dir, "registration", &detail);
        crate::output::print_log_warning(&detail);
    }
}

fn conflicting_session(pane: &Value) -> Option<&str> {
    let owner = pane["agent_session"]["agent"].as_str()?;
    (pane["agent_status"] == "unknown" && owner != super::request::AGENT).then_some(owner)
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

    fn assert_conflicting_session_only_while_unknown() {
        let mut pane =
            serde_json::json!({"agent_status":"unknown","agent_session":{"agent":"codex"}});
        assert_eq!(super::conflicting_session(&pane), Some("codex"));
        pane["agent_status"] = "working".into();
        assert_eq!(super::conflicting_session(&pane), None);
        pane["agent_status"] = "unknown".into();
        pane["agent_session"]["agent"] = "malvin".into();
        assert_eq!(super::conflicting_session(&pane), None);
        assert_eq!(super::conflicting_session(&serde_json::Value::Null), None);
    }

    fn serve_foreign_session_pane(listener: std::os::unix::net::UnixListener) {
        use std::io::{BufRead, BufReader, Write};
        std::thread::spawn(move || {
            for mut stream in listener.incoming().flatten() {
                let mut line = String::new();
                let _ = BufReader::new(&stream).read_line(&mut line);
                let req: serde_json::Value = serde_json::from_str(&line).unwrap_or_default();
                let reply = match req["method"].as_str() {
                    Some("agent.rename") => serde_json::json!({"id": req["id"], "error":
                        {"code": "agent_not_found", "message": "agent target p1 not found"}}),
                    Some("pane.get") => serde_json::json!({"id": req["id"], "result": {"pane":
                        {"agent_status": "unknown", "agent_session": {"agent": "codex"}}}}),
                    _ => serde_json::json!({"id": req["id"], "result": {"type": "ok"}}),
                };
                let _ = writeln!(stream, "{reply}");
            }
        });
    }

    #[test]
    fn bind_requests_and_foreign_saved_session_diagnostic() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("herdr.sock");
        serve_foreign_session_pane(std::os::unix::net::UnixListener::bind(&sock).unwrap());
        let env = crate::herdr::env::HerdrEnv {
            socket_path: sock,
            pane_id: "p1".into(),
        };
        emit_bind_reports(&env, Some("20261121_143419_sgj5tg58"), Some(dir.path()));
        let log = std::fs::read_to_string(dir.path().join("herdr.log")).unwrap();
        assert!(log.contains("herdr rename failed: agent_not_found"));
        assert!(log.contains("herdr registration failed: Herdr pane p1 still holds a saved codex"));
        assert_conflicting_session_only_while_unknown();
        assert_abs_path_string_falls_back_when_missing();
    }

    fn assert_abs_path_string_falls_back_when_missing() {
        let p = PathBuf::from("/no/such/malvin_herdr_path_xyz");
        let s = abs_path_string(&p).expect("fallback");
        assert!(s.contains("malvin_herdr_path_xyz"));
        let steps = bind_requests("pane", Some("20260804_140533_4gk60f1m"), None);
        assert!(steps.iter().any(|(p, _)| *p == "rename"));
        let _ = emit_bind_reports;
    }
}
