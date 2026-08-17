
use rand::Rng;
use serde_json::{json, Value};

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
    let suffix: u32 = rand::thread_rng().gen_range(0..1_000_000);
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
        clear_agent_authority, clear_metadata_teardown, rename_agent, report_agent,
        report_agent_session, report_metadata_sparse, AGENT, SOURCE,
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

        let rename = rename_agent("p1", "m4gk60f1m");
        assert_eq!(rename["method"], "agent.rename");
        assert_eq!(rename["params"]["target"], "p1");
        assert_eq!(rename["params"]["name"], "m4gk60f1m");
    }
}
