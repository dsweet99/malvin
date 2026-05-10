#[test]
fn kiss_stringify_tidy_flow_units() {
    let _ = stringify!(crate::cli::tidy_flow::TidyArgs);
    let _ = stringify!(crate::cli::tidy_flow::TidyAcpInput);
    let _ = stringify!(crate::cli::tidy_flow::prepare_tidy_prompt_store);
    let _ = stringify!(crate::cli::tidy_flow::TidyStartup);
    let _ = stringify!(crate::cli::tidy_flow::compose_tidy_prompt);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy_prompt);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy_acp);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy_and_learn);
    let _ = stringify!(crate::cli::tidy_flow::tidy_prompt_context);
    let _ = stringify!(crate::cli::tidy_flow::prepare_tidy_run);
    let _ = stringify!(crate::cli::tidy_flow::merge_tidy_timing);
    let _ = stringify!(crate::cli::tidy_flow::run_tidy);
}
