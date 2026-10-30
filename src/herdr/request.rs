use serde_json::{Value, json};

pub const SOURCE: &str = "herdr:malvin";
pub const AGENT: &str = "malvin";

#[must_use]
pub fn next_seq() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
}

#[must_use]
pub fn next_request_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let suffix: u32 = fastrand::u32(0..1_000_000);
    format!("{SOURCE}:{millis}:{suffix:06}")
}

#[must_use]
pub fn report_agent_session(
    pane_id: &str,
    agent_session_id: Option<&str>,
    agent_session_path: Option<&str>,
    seq: u64,
) -> Value {
    let mut params = json!({
        "pane_id": pane_id,
        "source": SOURCE,
        "agent": AGENT,
        "seq": seq,
    });
    if let Some(id) = agent_session_id {
        params["agent_session_id"] = json!(id);
    }
    if let Some(path) = agent_session_path {
        params["agent_session_path"] = json!(path);
    }
    envelope("pane.report_agent_session", params)
}

#[must_use]
pub fn rename_agent(target: &str, name: &str) -> Value {
    envelope(
        "agent.rename",
        json!({
            "target": target,
            "name": name,
        }),
    )
}

#[must_use]
pub fn report_agent(pane_id: &str, state: &str, agent_session_id: Option<&str>, seq: u64) -> Value {
    let mut params = json!({
        "pane_id": pane_id,
        "source": SOURCE,
        "agent": AGENT,
        "state": state,
        "seq": seq,
    });
    if let Some(id) = agent_session_id {
        params["agent_session_id"] = json!(id);
    }
    envelope("pane.report_agent", params)
}

#[must_use]
pub fn clear_agent_authority(pane_id: &str, seq: u64) -> Value {
    envelope(
        "pane.clear_agent_authority",
        json!({
            "pane_id": pane_id,
            "source": SOURCE,
            "seq": seq,
        }),
    )
}

#[must_use]
pub fn report_metadata_sparse(pane_id: &str, title: Option<&str>, seq: u64) -> Value {
    let mut params = json!({
        "pane_id": pane_id,
        "source": SOURCE,
        "display_agent": AGENT,
        "seq": seq,
    });
    if let Some(t) = title {
        params["title"] = json!(t);
    }
    if let Some(tokens) = crate::task_summary::metadata_tokens() {
        if let Some(label) = tokens.get("status").or_else(|| tokens.get("task")) {
            params["display_agent"] = label.clone();
        }
        params["tokens"] = tokens;
    }
    envelope("pane.report_metadata", params)
}

#[must_use]
pub fn clear_metadata_teardown(pane_id: &str, seq: u64) -> Value {
    envelope(
        "pane.report_metadata",
        json!({
            "pane_id": pane_id,
            "source": SOURCE,
            "seq": seq,
            "clear_display_agent": true,
            "clear_title": true,
            "tokens": {"task": null, "status": null},
        }),
    )
}

#[must_use]
pub fn release_agent(pane_id: &str, seq: u64) -> Value {
    envelope(
        "pane.release_agent",
        json!({
            "pane_id": pane_id,
            "source": SOURCE,
            "agent": AGENT,
            "seq": seq,
        }),
    )
}

fn envelope(method: &str, params: Value) -> Value {
    json!({
        "id": next_request_id(),
        "method": method,
        "params": params,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        AGENT, SOURCE, clear_agent_authority, clear_metadata_teardown, release_agent, rename_agent,
        report_agent, report_agent_session, report_metadata_sparse,
    };

    #[test]
    fn request_shapes_use_malvin_identity_and_methods() {
        let session = report_agent_session("p1", Some("run-id"), Some("/tmp/run"), 7);
        assert_eq!(session["method"], "pane.report_agent_session");
        assert_eq!(session["params"]["source"], SOURCE);
        assert_eq!(session["params"]["agent"], AGENT);
        assert_eq!(session["params"]["agent_session_id"], "run-id");
        assert_eq!(session["params"]["agent_session_path"], "/tmp/run");
        assert_eq!(session["params"]["seq"], 7);

        let working = report_agent("p1", "working", Some("run-id"), 8);
        assert_eq!(working["method"], "pane.report_agent");
        assert_eq!(working["params"]["state"], "working");

        let clear = clear_agent_authority("p1", 11);
        assert_eq!(clear["method"], "pane.clear_agent_authority");
        assert_eq!(clear["params"]["source"], SOURCE);
        assert_eq!(clear["params"]["pane_id"], "p1");

        let meta = report_metadata_sparse("p1", Some("title"), 10);
        assert_eq!(meta["method"], "pane.report_metadata");
        assert_eq!(meta["params"]["display_agent"], AGENT);
        assert_eq!(meta["params"]["title"], "title");

        let clear_meta = clear_metadata_teardown("p1", 12);
        assert_eq!(clear_meta["method"], "pane.report_metadata");
        assert_eq!(clear_meta["params"]["clear_display_agent"], true);
        assert_eq!(clear_meta["params"]["clear_title"], true);
        assert!(clear_meta["params"]["tokens"]["task"].is_null());
        assert!(clear_meta["params"]["tokens"]["status"].is_null());

        let release = release_agent("p1", 13);
        assert_eq!(release["method"], "pane.release_agent");
        assert_eq!(release["params"]["source"], SOURCE);
        assert_eq!(release["params"]["agent"], AGENT);
        assert_eq!(release["params"]["pane_id"], "p1");

        let rename = rename_agent("p1", "m4gk60f1m");
        assert_eq!(rename["method"], "agent.rename");
        assert_eq!(rename["params"]["target"], "p1");
        assert_eq!(rename["params"]["name"], "m4gk60f1m");
    }

    #[test]
    fn metadata_report_carries_task_and_status_tokens() {
        let _guard = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let run = dir.path().join("hash/run");
        std::fs::create_dir_all(&run).unwrap();
        crate::workspace::run_id::set_active_run_dir(Some(run.clone()));
        std::fs::write(
            run.join("request_summary.txt"),
            "Add cheap request summaries",
        )
        .unwrap();
        let task_meta = report_metadata_sparse("p1", None, 9);
        assert_eq!(
            task_meta["params"]["display_agent"],
            "Add cheap request summaries"
        );
        std::fs::write(run.join("status_summary.txt"), "Reading the plan").unwrap();
        let meta = report_metadata_sparse("p1", None, 10);
        assert_eq!(meta["params"]["display_agent"], "Reading the plan");
        assert_eq!(
            meta["params"]["tokens"]["task"],
            "Add cheap request summaries"
        );
        assert_eq!(meta["params"]["tokens"]["status"], "Reading the plan");
        std::fs::write(run.join("status_summary.txt"), "Return only that summary").unwrap();
        let invalid_status = report_metadata_sparse("p1", None, 11);
        assert_eq!(
            invalid_status["params"]["display_agent"],
            "Add cheap request summaries"
        );
        crate::workspace::run_id::set_active_run_dir(None);
        assert_eq!(
            report_metadata_sparse("p1", None, 12)["params"]["display_agent"],
            AGENT
        );
    }
}
