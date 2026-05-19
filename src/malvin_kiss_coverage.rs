#[test]
fn kiss_stringify_crate_root_units() {
    let _ = stringify!(crate::time_format::timestamp_now_string);
    let _ = stringify!(crate::stdout_log_path::set_stdout_log_path);
    let _ = stringify!(crate::stdout_log_path::clone_stdout_log_path);
    let _ = stringify!(crate::artifacts::create_run_artifacts);
    let _ = stringify!(crate::artifacts::create_run_artifacts_from_text);
    let _ = stringify!(crate::artifacts::create_kpop_run_artifacts);
    let _ = stringify!(crate::artifacts::work_dir_for_path);
    let _ = stringify!(crate::artifacts::resolve_user_at_path);
    let _ = stringify!(crate::artifacts::resolve_at_file);
    let _ = stringify!(crate::artifacts::resolve_user_request);
    let _ = stringify!(crate::learn_gate::should_run_learn_check);
    let _ = stringify!(crate::acp::session_types::AcpSessionInner);
    let _ = stringify!(crate::kpop_multiturn_prompts::SmokeKpopBuilder);
    let _ = stringify!(crate::child_health::macos::status_char_hint);
    let _ = stringify!(crate::output::timestamp_now_string);
    let _ = stringify!(crate::output::set_stdout_log_path);
}
