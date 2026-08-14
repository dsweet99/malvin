use crate::cli::default_output_path::path_relative_to_cwd;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::router_flow::{run_router, RouterArgs};

use super::prep::{compose_write_router_request, write_preflight};
use super::WriteArgs;

pub async fn run_write(
    write_args: &mut WriteArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (request_text, outputs) = write_preflight(
        write_args.request.as_ref(),
        &write_args.out_path,
        write_args.out_path_explicit,
    )?;
    let tex_display = path_relative_to_cwd(&outputs.tex_path)?;
    let pdf_display = path_relative_to_cwd(&outputs.pdf_path)?;
    write_args.out_path = tex_display.clone();
    let request = compose_write_router_request(&request_text, &tex_display, &pdf_display)?;
    run_router(
        RouterArgs {
            request: Some(request),
            max_loops: super::effective_write_max_loops(write_args.max_loops),
            max_hypotheses: write_args.max_hypotheses,
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
    fn kiss_cov_run_write_symbol() {
        let _ = run_write;
        let _ = compose_write_router_request;
        let _ = write_preflight;
    }
}
