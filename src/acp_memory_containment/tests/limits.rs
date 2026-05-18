#[test]
fn mod_helpers_surface() {
    use crate::acp_memory_containment::{
        ContainmentHandle, remove_containment_handle, write_containment_unavailable_warn,
    };

    let _ = crate::acp_memory_containment::half_physical_memory_bytes();
    remove_containment_handle(ContainmentHandle::Inactive);
    let mut buf = Vec::new();
    write_containment_unavailable_warn(&mut buf).expect("write");
    assert!(
        String::from_utf8(buf)
            .expect("utf8")
            .contains("ACP memory containment unavailable")
    );
}

#[test]
fn inactive_containment_never_reports_oom() {
    let c = crate::acp_memory_containment::AcpMemoryContainment::inactive();
    assert!(!c.memory_limit_exceeded());
}
