use crate::cli::default_output_path::path_relative_to_cwd;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::router_flow::{run_router, RouterArgs};

use super::prep::{compose_delight_router_request, delight_preflight, resolve_delight_guidance};
use super::DelightArgs;

/// Run `malvin delight` as a composed request into the default router workflow.
pub async fn run_delight(
    delight: &mut DelightArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (resolved_out_path, _work_dir) = delight_preflight(&delight.out_path)?;
    delight.out_path = path_relative_to_cwd(&resolved_out_path)?;
    let guidance = resolve_delight_guidance(delight.guidance.as_ref())?;
    let request = compose_delight_router_request(&delight.out_path, guidance.as_deref());
    run_router(
        RouterArgs {
            request: Some(request),
            max_loops: super::effective_delight_max_loops(delight.max_loops),
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
    fn kiss_cov_run_delight_symbol() {
        let _ = run_delight;
        let _ = compose_delight_router_request;
        let _ = delight_preflight;
        let _ = resolve_delight_guidance;
    }
}
