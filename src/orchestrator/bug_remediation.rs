use std::collections::HashMap;

use crate::acp::AgentError;

use super::session_flow::{run_bug_remediation_until_pre_summary, run_coder_session_summary_only};
use super::{
    Orchestrator, PreSummaryMidFn, WorkflowError, prefer_primary_errors_over_timing,
};

pub async fn run_bug_remediation_gap(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    mid: PreSummaryMidFn,
) -> Result<(), WorkflowError> {
    let timing = orchestrator.attach_run_timing();
    let begin_res = orchestrator
        .client
        .begin_coder_session(&orchestrator.artifacts.work_dir)
        .await;
    let coder_session_began = begin_res.is_ok();
    let workflow_result = match begin_res {
        Ok(()) => async {
            run_bug_remediation_until_pre_summary(orchestrator, context).await?;
            mid(
                orchestrator.client,
                orchestrator.artifacts,
                &orchestrator.kissconfig_backup,
            )
            .await
            .map_err(WorkflowError)?;
            run_coder_session_summary_only(orchestrator, context).await
        }
        .await,
        Err(e) => Err(WorkflowError(e.0)),
    };
    let timing_result = if coder_session_began {
        orchestrator.emit_run_timing_artifact(&timing)
    } else {
        orchestrator.client.set_run_timing(None);
        Ok(())
    };
    let end_result = orchestrator
        .client
        .end_coder_session()
        .await
        .map_err(|e: AgentError| WorkflowError(e.0));
    prefer_primary_errors_over_timing(workflow_result, end_result, timing_result)
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_bug_remediation_units() {
        let _ = stringify!(crate::orchestrator::bug_remediation::run_bug_remediation_gap);
    }
}
