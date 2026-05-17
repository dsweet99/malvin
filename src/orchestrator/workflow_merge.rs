use super::WorkflowError;

/// # Errors
///
/// Returns the primary run error when only the run fails; otherwise the restore error, or both when both fail.
pub fn merge_string_run_and_restore(
    run_result: Result<(), String>,
    restore_result: Result<(), String>,
) -> Result<(), String> {
    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(run_err), Ok(())) => Err(run_err),
        (Ok(()), Err(restore_err)) => Err(restore_err),
        (Err(run_err), Err(restore_err)) => Err(format!(
            "{run_err}; workspace session restore failed: {restore_err}"
        )),
    }
}

/// # Errors
///
/// Returns the primary run error when only the run fails; otherwise the restore error, or both when both fail.
pub fn merge_workflow_run_and_restore(
    run_result: Result<(), WorkflowError>,
    restore_result: Result<(), WorkflowError>,
) -> Result<(), WorkflowError> {
    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(run_err), Ok(())) => Err(run_err),
        (Ok(()), Err(restore_err)) => Err(restore_err),
        (Err(run_err), Err(restore_err)) => Err(WorkflowError(format!(
            "{}; workspace session restore failed: {}",
            run_err.0, restore_err.0
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_prefers_run_error_when_both_fail() {
        let err = merge_workflow_run_and_restore(
            Err(WorkflowError("run failed".to_string())),
            Err(WorkflowError("restore failed".to_string())),
        )
        .expect_err("both failed");
        assert!(err.0.contains("run failed"));
        assert!(err.0.contains("restore failed"));
    }

    #[test]
    fn merge_string_run_and_restore_labels_restore_failure() {
        let err = merge_string_run_and_restore(Err("run".to_string()), Err("restore".to_string()))
            .expect_err("both failed");
        assert!(err.contains("workspace session restore failed"));
    }

    #[test]
    fn kiss_stringify_workflow_merge_units() {
        let _ = stringify!(super::merge_workflow_run_and_restore);
        let _ = stringify!(super::merge_string_run_and_restore);
    }
}
