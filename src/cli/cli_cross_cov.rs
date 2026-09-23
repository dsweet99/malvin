#[test]
fn smoke_format_gate_failures_non_empty() {
    let ws = super::format_workspace_gate_failure("malvin -g", "gate failed");
    assert!(ws.contains("gate failed"));
    assert!(ws.contains("malvin -g"));
}
