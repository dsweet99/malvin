use malvin::artifacts::RunArtifacts;
use malvin::format_logs_dir;
use malvin::output::{MALVIN_WHO, print_stdout_line};

pub(crate) const ROUTER_SUMMARY_PREFIX: &str = "SUMMARY: ";

#[must_use]
pub(crate) fn format_router_summary_line(num_iter: usize, logs_dir: &str) -> String {
    format!("{ROUTER_SUMMARY_PREFIX}num_iter = {num_iter} logs_dir = {logs_dir}")
}

pub(crate) fn emit_router_summary_line(
    artifacts: &RunArtifacts,
    num_iter: usize,
) -> Result<(), String> {
    let logs_dir = format_logs_dir(&artifacts.run_dir)?;
    print_stdout_line(
        MALVIN_WHO,
        &format_router_summary_line(num_iter, &logs_dir),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ROUTER_SUMMARY_PREFIX, emit_router_summary_line, format_router_summary_line};
    use malvin::output::{STDOUT_LOG_TEST_LOCK, set_stdout_log_path};

    #[test]
    fn format_router_summary_line_matches_requested_shape() {
        let line = format_router_summary_line(3, "/tmp/logs/run");
        assert_eq!(line, "SUMMARY: num_iter = 3 logs_dir = /tmp/logs/run");
        assert!(line.starts_with(ROUTER_SUMMARY_PREFIX));
    }

    #[test]
    fn emit_router_summary_line_writes_stdout_log() {
        let _lock = STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        malvin::test_utils::with_isolated_home(|_| {
            let tmp = tempfile::tempdir().expect("tempdir");
            let artifacts =
                malvin::artifacts::create_run_artifacts_from_text("summary", Some(tmp.path()))
                    .expect("art");
            let log_path = tmp.path().join("stdout.log");
            set_stdout_log_path(Some(log_path.clone()));
            emit_router_summary_line(&artifacts, 2).expect("emit");
            set_stdout_log_path(None);
            let text = std::fs::read_to_string(&log_path).expect("read");
            assert!(
                text.contains("SUMMARY: num_iter = 2 logs_dir = "),
                "stdout.log missing SUMMARY line: {text:?}"
            );
        });
    }
}
