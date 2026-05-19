#[test]
fn mod_helpers_surface() {
    use crate::acp_memory_containment::{ContainmentHandle, remove_containment_handle};

    let _ = crate::acp_memory_containment::half_physical_memory_bytes();
    remove_containment_handle(ContainmentHandle::Inactive);
}

#[test]
fn inactive_containment_never_reports_oom() {
    let c = crate::acp_memory_containment::AcpMemoryContainment::inactive();
    assert!(!c.memory_limit_exceeded());
}
