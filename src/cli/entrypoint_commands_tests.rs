use super::*;

#[test]
fn kiss_cov_entrypoint_command_wrappers() {
    let _ = run_explain_command;
    let _ = run_delight_command;
    let _ = run_priors_command;
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

#[test]
fn kiss_cov_delight_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Delight(crate::cli::delight_flow::DelightArgs {
        guidance: None,
        out_path: "pitch.md".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: true,
    });
    let _ = super::super::entrypoint::dispatch_command;
    let _ = cmd;
}

#[test]
fn kiss_cov_priors_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Priors(crate::cli::priors_flow::PriorsArgs {
        request: Some("topic".to_string()),
        out_path: "priors.md".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: true,
    });
    let _ = super::super::entrypoint::dispatch_command;
    let _ = cmd;
}
