pub(crate) fn child_health_timeout_error(
    outcome: crate::child_health::SilenceHealthOutcome,
) -> Option<String> {
    match outcome {
        crate::child_health::SilenceHealthOutcome::ChildNotRunning => {
            Some("acp child process is not running".to_string())
        }
        crate::child_health::SilenceHealthOutcome::ChildZombie => {
            Some("acp child process is zombie".to_string())
        }
        crate::child_health::SilenceHealthOutcome::StillBusyExtendWait => None,
        crate::child_health::SilenceHealthOutcome::AppearsHung => {
            Some("acp child process appears hung".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::child_health_timeout_error;

    #[test]
    fn child_health_timeout_error_fixed_strings() {
        assert_eq!(
            child_health_timeout_error(crate::child_health::SilenceHealthOutcome::ChildNotRunning)
                .as_deref(),
            Some("acp child process is not running")
        );
        assert!(child_health_timeout_error(
            crate::child_health::SilenceHealthOutcome::StillBusyExtendWait
        )
        .is_none());
    }
}
