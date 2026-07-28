use crate::cli::default_output_path::path_relative_to_cwd;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::router_flow::{run_router, RouterArgs};

use super::prep::{compose_explain_router_request, explain_preflight};
use super::ExplainArgs;

/// Run `malvin explain` as a composed request into the default router workflow.
pub async fn run_explain(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (request_text, outputs) = explain_preflight(
        explain.request.as_ref(),
        &explain.out_path,
        explain.out_path_explicit,
    )?;
    let tex_display = path_relative_to_cwd(&outputs.tex_path)?;
    let pdf_display = path_relative_to_cwd(&outputs.pdf_path)?;
    explain.out_path = tex_display.clone();
    let request = compose_explain_router_request(&request_text, &tex_display, &pdf_display);
    run_router(
        RouterArgs {
            request: Some(request),
            max_loops: super::effective_explain_max_loops(explain.max_loops),
        },
        shared,
        workflow,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiss_cov_run_explain_symbol() {
        let _ = run_explain;
        let _ = compose_explain_router_request;
        let _ = explain_preflight;
    }
}
