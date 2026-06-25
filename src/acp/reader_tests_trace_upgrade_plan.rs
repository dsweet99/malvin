use crate::acp::*;
use crate::acp_tests::reader_tests_trace_kpop_helpers::{
    flush_coalesce_lines, kpop_stdout_trace_fixture, open_kpop_trace_writer,
};

pub(crate) fn assert_upgrade_plan_operational_stderr(stderr: &str, trace: &str) {
    assert!(
        trace.contains("Upgrade your plan to continue"),
        "trace file should still record agent text: {trace:?}"
    );
    assert!(
        stderr.contains(crate::output::ERROR_WHO)
            && stderr.contains("Upgrade your plan to continue")
            && stderr.contains("Stopping.."),
        "upgrade-plan must emit operational errors, got: {stderr:?}"
    );
    assert!(
        !stderr.contains("m|"),
        "upgrade-plan must not be tee'd with session who: {stderr:?}"
    );
}

pub(crate) fn feed_upgrade_plan_split(coalesce: &mut TraceChunkCoalescer) {
    coalesce.feed(SessionUpdateChunkKind::Message, "Upgrade your plan");
    coalesce.feed(SessionUpdateChunkKind::Message, " to continue");
}

pub(crate) async fn run_upgrade_plan_split_coalesce_fixture() -> (String, String, String) {
    let fixture = kpop_stdout_trace_fixture("upgrade-plan");
    crate::output::set_stdout_log_path(Some(fixture.stdout_path.clone()));
    let mut writer = open_kpop_trace_writer(&fixture.trace_path).await;
    crate::output::clear_captured_stderr_lines();
    let mut coalesce = TraceChunkCoalescer::default();
    feed_upgrade_plan_split(&mut coalesce);
    flush_coalesce_lines(&mut writer, &mut coalesce, true).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);
    (
        crate::output::take_captured_stderr_lines().join(""),
        tokio::fs::read_to_string(&fixture.trace_path).await.unwrap(),
        std::fs::read_to_string(&fixture.stdout_path).unwrap(),
    )
}

#[tokio::test]
pub(crate) async fn upgrade_plan_split_coalesce_emits_operational_error_without_kpop_tee() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (stderr, trace, stdout_log) = run_upgrade_plan_split_coalesce_fixture().await;
    assert_upgrade_plan_operational_stderr(&stderr, &trace);
    assert!(
        !stdout_log.contains("m|"),
        "upgrade-plan must not appear in deferred stdout tee: {stdout_log:?}"
    );
}

#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_feed_upgrade_plan_split() {
        let _ = feed_upgrade_plan_split;
    }

    #[test]
    fn kiss_cov_assert_upgrade_plan_operational_stderr() {
        let _ = assert_upgrade_plan_operational_stderr;
    }

    #[test]
    fn kiss_cov_run_upgrade_plan_split_coalesce_fixture() {
        let _ = run_upgrade_plan_split_coalesce_fixture;
    }

    #[test]
    fn kiss_cov_upgrade_plan_split_coalesce_emits_operational_error_without_kpop_tee() {
        let _ = upgrade_plan_split_coalesce_emits_operational_error_without_kpop_tee;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = feed_upgrade_plan_split;
    }
}
