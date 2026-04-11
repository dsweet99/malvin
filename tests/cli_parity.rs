//! Behavioral constraints for the CLI.

#[test]
fn max_loops_zero_must_not_be_clamped_to_one() {
    let max_loops = 0_usize;
    let iterations = (1..=max_loops).count();
    assert_eq!(iterations, 0, "range(1..=max_loops) yields zero iterations when max_loops is 0");
    let main_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"));
    assert!(
        !main_rs.contains("args.max_loops.max(1)"),
        "main.rs must not clamp max_loops with .max(1); that breaks the intended zero-iteration behavior"
    );
    let cli_rs = concat!(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/args.rs")),
    );
    assert!(
        !cli_rs.contains("max_loops.max(1)"),
        "src/cli must not clamp max_loops with .max(1); that breaks the intended zero-iteration behavior"
    );
}

const fn agent_sources_for_snapshot() -> &'static str {
    concat!(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/ops_body.inc")),
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/client_impl.inc")),
    )
}

#[test]
fn agent_client_must_apply_force_when_invoking_acp() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        !snapshot.contains("let _ = self.force;"),
        "force is stored on AgentClient but discarded before spawn; --force should be passed to cursor-agent when force is true"
    );
    assert!(
        snapshot.contains("force: client.io.force"),
        "spawn must pass `agent --force` via client.io.force"
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
        "spawn must pass model into AcpSpawnArgs"
    );
}
