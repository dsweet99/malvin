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
fn default_cli_model_is_composer_2() {
    let shared = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/shared_opts.rs"));
    assert!(
        shared.contains("const DEFAULT_CLI_MODEL")
            && shared.contains("\"composer-2\"")
            && shared.contains("default_value = DEFAULT_CLI_MODEL"),
        "default `--model` must remain composer-2 via DEFAULT_CLI_MODEL unless intentionally changed"
    );
    let models = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/models_cmd.rs"));
    assert!(
        models.contains("DEFAULT_CLI_MODEL")
            && models.contains("Default model in malvin: {DEFAULT_CLI_MODEL}"),
        "`malvin models` footer must use DEFAULT_CLI_MODEL (same string as SharedOpts default)"
    );
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

#[test]
fn agent_client_must_apply_tee_mode_when_invoking_acp() {
    let snapshot = agent_sources_for_snapshot();
    assert!(
        snapshot.contains("tee_trace_stdout: !client.io.no_tee"),
        "spawn must pass CLI tee mode into AcpSpawnArgs so trace lines can stream to stdout when tee is on"
    );
}

#[test]
fn upgrade_plan_message_must_not_be_eprint_twice() {
    let client_impl = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/client_impl.inc"));
    let cli_mod = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs"));
    let client_eprints_on_upgrade = client_impl.contains("agent_string_is_upgrade_plan")
        && client_impl.contains("eprintln!(\"{last_error}\")");
    let cli_eprints_run_error = cli_mod.contains("eprintln!(\"{e}\")");
    assert!(
        !(client_eprints_on_upgrade && cli_eprints_run_error),
        "upgrade-plan failures eprintln in client_impl and the CLI entrypoint prints AgentError again; stderr duplicates the same message"
    );
}
