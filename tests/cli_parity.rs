//! Behavioral parity with Python `agent_coding/malvin` and project constraints.

#[test]
fn max_loops_zero_must_not_be_clamped_to_one() {
    let max_loops = 0_usize;
    let iterations = (1..=max_loops).count();
    assert_eq!(iterations, 0, "Python uses range(1, max_loops + 1): zero iterations when max_loops is 0");
    let main_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"));
    assert!(
        !main_rs.contains("args.max_loops.max(1)"),
        "main.rs must not clamp max_loops with .max(1); that breaks parity with Python"
    );
}

const fn agent_sources_for_snapshot() -> &'static str {
    concat!(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/client.rs")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/ops.rs")),
    )
}

#[test]
fn agent_client_must_apply_force_when_invoking_acp() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        !snapshot.contains("let _ = self.force;"),
        "force is stored on AgentClient but discarded before spawn; Python passes --force to cursor-agent when force is true"
    );
    assert!(
        snapshot.contains("force: client.io.force"),
        "spawn must pass `agent --force` via client.io.force (parity with Python cursor-agent)"
    );
}

#[test]
fn agent_client_must_apply_model_when_invoking_acp_or_drop_cli_option() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        !snapshot.contains("let _ = self.model;"),
        "model is accepted on the CLI but discarded before spawn; wire through ACP or document-only at the type level"
    );
    assert!(
        snapshot.contains("model_opt"),
        "spawn must pass model into AcpSpawnArgs (parity with Python `--model`)"
    );
}
