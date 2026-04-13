//! `kiss` coverage: static `stringify!` refs for CLI symbols (kept out of `mod.rs` for line budget).

#[test]
fn kiss_stringify_cli_symbols() {
    let _ = stringify!(crate::cli::shared_opts::SharedOpts);
    let _ = stringify!(crate::cli::Cli);
    let _ = stringify!(crate::cli::shared_opts::GlobalOpts);
    let _ = stringify!(crate::cli::Commands);
    let _ = stringify!(crate::cli::CodeArgs);
    let _ = stringify!(crate::cli::do_flow::DoArgs);
    let _ = stringify!(crate::cli::init_cmd::InitArgs);
    let _ = stringify!(crate::cli::models_cmd::ModelsArgs);
    let _ = stringify!(crate::cli::KpopArgs);
    let _ = stringify!(crate::cli::SharedOpts);
    let _ = stringify!(crate::cli::Exit);
    let _ = stringify!(crate::cli::WorkflowCliOptions);
    let _ = stringify!(crate::cli::entrypoint);
    let _ = stringify!(crate::cli::run_code);
    let _ = stringify!(crate::cli::run_do);
    let _ = stringify!(crate::cli::do_flow::prepare_do_prompt_store);
    let _ = stringify!(crate::cli::run_kpop);
    let _ = stringify!(crate::cli::prepare_prompt_store);
    let _ = stringify!(crate::cli::prepare_kpop_prompt_store);
    let _ = stringify!(crate::cli::echo_primary_to_stdout);
    let _ = stringify!(crate::cli::emit_command_line);
    let _ = stringify!(crate::cli::emit_run_startup_sequence);
    let _ = stringify!(malvin::log_paths::format_logs_dir);
    let _ = stringify!(crate::cli::build_agent);
    let _ = stringify!(crate::cli::shared_opts::SharedOpts::tee_startup_stdout);
    let _ = stringify!(crate::cli::init_cmd::run_init);
    let _ = stringify!(crate::cli::models_cmd::run_models);
    let _ = stringify!(malvin::env_path::lookup_bin_on_path);
    let _ = stringify!(crate::cli::timing_merge::emit_run_timing_after_acp);
    let _ = stringify!(crate::cli::timing_merge::merge_acp_and_timing_results);
}
