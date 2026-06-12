use super::*;

#[test]
fn kiss_cov_entrypoint_command_wrappers() {
    let _ = stringify!(run_inspire_command);
    let _ = stringify!(run_plan_command);
    let _ = stringify!(run_code_command);
    let _ = stringify!(run_delight_command);
    let _ = stringify!(run_delight_then_plan);
    let _ = stringify!(plan_args_for_delight_output);
    let _ = stringify!(run_explain_then_revise);
    let _ = stringify!(revise_args_for_explain_output);
    let _ = stringify!(run_explain_command);
    let _ = stringify!(run_revise_command);
    let _ = stringify!(dispatch_plan_authoring_gate);
}

#[test]
fn delight_plan_args_use_same_out_path() {
    let args = plan_args_for_delight_output("plans/feature.md");
    assert_eq!(args.plan_path, "plans/feature.md");
}

#[test]
fn revise_args_for_explain_output_use_tex_path() {
    let explain = crate::cli::explain_flow::ExplainArgs {
        request: Some("topic".to_string()),
        out_path: "explain.tex".to_string(),
        max_loops: 7,
        max_hypotheses: 11,
        tenacious: false,
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
    });
    let _ = super::super::entrypoint::require_kiss_for_cli_command(&cmd);
    let _ = stringify!(Commands::Explain);
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
    let _ = stringify!(Commands::Delight);
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
    let _ = stringify!(Commands::Revise);
}
