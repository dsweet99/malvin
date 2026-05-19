use crate::acp_memory_containment::{CONTAINMENT_UNAVAILABLE_WARN, emit_containment_unavailable_warn};
use crate::test_stderr_capture::capture_stderr_output;

#[test]
fn emit_containment_unavailable_warn_matches_malvin_log_line() {
    let stderr = capture_stderr_output(emit_containment_unavailable_warn);
    assert!(stderr.contains(CONTAINMENT_UNAVAILABLE_WARN));
    assert!(stderr.contains("[warning"));
}
