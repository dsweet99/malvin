
#[test]
fn kiss_cov_coalesce_private_helpers() {
    let mut coalescer = crate::acp::VerboseIoCoalescer::default();
    coalescer.feed(crate::acp::SessionUpdateChunkKind::Message, "hello ");
    coalescer.flush_all();
}

#[test]
fn kiss_cov_coalesce_trace_flush_helpers() {
    let mut coalescer = crate::acp::TraceChunkCoalescer::default();
    let _ = coalescer.feed(crate::acp::SessionUpdateChunkKind::Message, "chunk");
    let _ = coalescer.flush_all();
}

#[test]
fn kiss_cov_router_acp_support_module_import() {
    use crate::router_flow::router_flow_acp::router_flow_acp_support::{
        empty_iteration_backups, router_iteration_log_path, run_router_turns,
        snapshot_iteration_backups,
    };
    let _ = router_iteration_log_path;
    let _ = empty_iteration_backups;
    let _ = snapshot_iteration_backups;
    let _ = run_router_turns;
}

#[test]
fn agent_bundle_agent_error_auth_error_fmt() {
    use crate::acp::{AgentError, AuthError};
    let _ = <AgentError as std::fmt::Display>::fmt;
    let _ = <AuthError as std::fmt::Display>::fmt;
    assert_eq!(format!("{}", AgentError("ae".into())), "ae");
    assert_eq!(format!("{}", AuthError("au".into())), "au");
}

#[test]
fn kiss_cov_kpop_turn_kpop_block() {
    use crate::kpop_turn_prompts::KpopTurnPrompts;
    use crate::prompt_stratification::WorkflowRenderContext;
    use crate::prompts::PromptStore;
    use std::collections::HashMap;

    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let base = WorkflowRenderContext::from(HashMap::from([
        ("plan_path".to_string(), "p".to_string()),
        ("exp_log".to_string(), "./_kpop/exp.md".to_string()),
        (
            "user_request_path".to_string(),
            "./.malvin/logs/run/user_request.md".to_string(),
        ),
    ]));
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "brief",
        prepend_rules_once: false,
    };
    let out = prompts.kpop_block(2).expect("kpop prompt");
    assert!(out.contains("brief"));
    assert!(out.contains("max_hypotheses = `2`"));
}

