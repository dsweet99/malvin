//! External kiss witnesses for `workflow_kpop_render` privates.

#[test]
fn kiss_witness_render_kpop_program() {
    let _: Option<super::RenderKpopProgram> = None;
    let _ = super::render_kpop_program_request;
    let _ = super::render_kpop_program_request_creative;
    let _ = super::render_kpop_program_request_with_template;
}
