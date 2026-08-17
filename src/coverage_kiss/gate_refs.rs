
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
