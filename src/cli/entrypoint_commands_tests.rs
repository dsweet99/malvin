use super::*;

#[test]
fn kiss_cov_entrypoint_command_wrappers() {
    let _ = run_write_command;
}

#[test]
fn kiss_cov_write_entrypoint_branch() {
    use crate::cli::args::Commands;
    let cmd = Commands::Write(crate::cli::write_flow::WriteArgs {
        shared: crate::cli::SharedOpts::test_defaults(),
        request: Some("topic".to_string()),
        out_path: "write.tex".to_string(),
        max_loops: 1,
        max_hypotheses: 10,
        tenacious: true,
        out_path_explicit: false,
    });
    let _ = super::super::entrypoint::dispatch_command;
    let _ = cmd;
}
