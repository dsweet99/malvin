//! Cross-module behavioral smokes and static refs for orchestrator kiss per-file coverage.

use super::fail_on_abort_for_artifacts;

#[test]
fn smoke_fail_on_abort_for_artifacts_ok_when_no_abort() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = crate::artifacts::create_run_artifacts_from_text("orch-read", Some(tmp.path()))
        .expect("artifacts");
    fail_on_abort_for_artifacts(&artifacts).expect("no abort");
}
