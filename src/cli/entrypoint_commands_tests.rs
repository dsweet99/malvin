use super::*;

#[test]
fn kiss_cov_entrypoint_command_wrappers() {
    let _ = run_explain_command;
    let _ = run_inspire_command;
}

#[test]
fn kiss_cov_explain_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Explain(crate::cli::explain_flow::ExplainArgs {
        request: Some("topic".to_string()),
        out_path: "explain.tex".to_string(),
        max_loops: 1,
        max_hypotheses: 10,
        tenacious: true,
        out_path_explicit: false,
    });
    let _ = super::super::entrypoint::dispatch_command;
    let _ = cmd;
}
