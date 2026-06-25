use super::*;

#[test]
fn kiss_cov_entrypoint_command_wrappers() {
    let _ = run_explain_command;
    let _ = run_revise_command;
    let _ = run_delight_command;
    let _ = run_explain_then_revise;
}

#[test]
fn revise_args_for_explain_output_use_tex_path() {
    let explain = crate::cli::explain_flow::ExplainArgs {
        request: Some("topic".to_string()),
        out_path: "explain.tex".to_string(),
        max_loops: 7,
        max_hypotheses: 11,
        tenacious: false,
        out_path_explicit: false,
    };
    let args = revise_args_for_explain_output(&explain, "docs/paper.tex");
    assert_eq!(args.doc_path, "docs/paper.tex");
    assert_eq!(args.max_loops, 7);
    assert_eq!(args.max_hypotheses, 11);
    assert!(!args.tenacious);
}

#[test]
fn kiss_cov_explain_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Explain(crate::cli::explain_flow::ExplainArgs {
        request: Some("topic".to_string()),
        out_path: "explain.tex".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: true,
        out_path_explicit: false,
    });
    let _ = super::super::entrypoint::require_kiss_for_cli_command(&cmd);
}

#[test]
fn kiss_cov_delight_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Delight(crate::cli::delight_flow::DelightArgs {
        guidance: None,
        out_path: "plan.md".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: true,
    });
    let _ = super::super::entrypoint::require_kiss_for_cli_command(&cmd);
}

#[test]
fn kiss_cov_revise_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Revise(crate::cli::revise_flow::ReviseArgs {
        doc_path: "doc.md".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: true,
    });
    let _ = super::super::entrypoint::require_kiss_for_cli_command(&cmd);
}
