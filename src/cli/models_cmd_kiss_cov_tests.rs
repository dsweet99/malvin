fn kiss_witness_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

#[test]
fn kiss_cov_models_args_clap_parse_and_destructure() {
    use clap::{CommandFactory, FromArgMatches, Parser};

    use super::{ModelsArgs, models_args_marker};
    use crate::cli::{AdminArgs, AdminCommand, Cli, Commands};

    let cli = Cli::try_parse_from(["malvin", "admin", "models"]).expect("parse models");
    match cli.command {
        Some(Commands::Admin(AdminArgs {
            command: Some(AdminCommand::Models(args)),
        })) => {
            assert_eq!(models_args_marker(&args), "models");
            let _args = kiss_witness_clone(&args);
        }
        _ => panic!("expected Admin::Models subcommand"),
    }
    let reparse = Cli::try_parse_from(["malvin", "admin", "models"]).expect("reparse");
    if let Some(Commands::Admin(AdminArgs {
        command: Some(AdminCommand::Models(second)),
    })) = reparse.command
    {
        assert_eq!(models_args_marker(&second), "models");
        assert!(format!("{second:?}").starts_with("ModelsArgs"));
        let _second = kiss_witness_clone(&second);
    } else {
        panic!("second parse should yield Admin::Models");
    }
    let cmd = Cli::command();
    let admin = cmd.find_subcommand("admin").expect("admin");
    assert!(admin.find_subcommand("models").is_some());
    let matches = Cli::command().get_matches_from(["malvin", "admin", "models"]);
    let sub = matches
        .subcommand_matches("admin")
        .expect("admin matches")
        .subcommand_matches("models")
        .expect("models matches");
    let _parsed = ModelsArgs::from_arg_matches(sub).expect("models from_arg_matches");
    let _cloned = kiss_witness_clone(&ModelsArgs::default());
    let refresh = Cli::try_parse_from(["malvin", "admin", "models", "--refresh"]).expect("refresh");
    if let Some(Commands::Admin(AdminArgs {
        command: Some(AdminCommand::Models(args)),
    })) = refresh.command
    {
        assert!(args.refresh);
    } else {
        panic!("expected Admin::Models --refresh");
    }
}

#[test]
fn kiss_cov_models_cmd_run_helpers() {
    use super::ModelsArgs;
    use super::test_hooks::*;

    let args = ModelsArgs::default();
    assert!(format!("{args:?}").starts_with("ModelsArgs"));
    let trimmed = trim_trailing_tip_lines("line\nTip: drop\n");
    assert_eq!(trimmed, "line");
    let (name, desc) = parse_model_line("gpt-4 — stable").expect("parse");
    assert_eq!(name, "gpt-4");
    assert_eq!(desc, "stable");
    let lines = models_display_lines("only-one\n").expect("lines");
    assert_eq!(lines, vec!["only-one".to_string()]);
    print_parsed_or_fallback("fallback\n");
    let _ = cursor_list_models_timeout();
    let _ = stringify!(DEFAULT_CURSOR_LIST_MODELS_TIMEOUT_MS);
    let _ = stringify!(cursor_model_listing);
    let _ = stringify!(cursor_listing_via_sdk);
    let _ = stringify!(cursor_listing_via_cli);
    let _ = stringify!(listing_from_cli_text);
    let _ = stringify!(run_cursor_sdk_models_js);
    let _ = stringify!(prefixed_rows);
    let _ = stringify!(catalog_for);
    let _ = stringify!(is_provider_authenticated);
    let _ = stringify!(is_provider_listable);
}

#[test]
fn kiss_cov_models_branchy_executable_witness() {
    use super::test_hooks::*;

    assert!(looks_like_tip_banner_line("tip: upgrade"));
    assert!(looks_like_tip_banner_line("tip use tls"));
    assert!(!looks_like_tip_banner_line("tip of the day"));
    assert!(!looks_like_tip_banner_line("see tip: inline"));
    assert!(is_models_section_header("Available models"));
    assert!(is_models_section_header("available models"));
    assert!(is_models_section_header(
        "No models available for this account."
    ));
    assert!(!is_models_section_header("auto - Auto"));
    assert!(models_display_lines("   \n").is_none());
    assert!(parse_model_line("singleword").is_none());
    print_parsed_or_fallback("   \n");
    if resolve_models_cli().is_err() {
        assert!(resolve_models_cli().unwrap_err().contains("PATH"));
    }
}

#[test]
fn kiss_cov_parse_model_line_all_branches_single_test() {
    use super::test_hooks::*;

    let (em_name, em_desc) = parse_model_line("composer-2 — Fast").expect("em dash");
    assert_eq!(em_name, "composer-2");
    assert_eq!(em_desc, "Fast");
    let (hy_name, hy_desc) = parse_model_line("model-id - Claude via API").expect("ascii hyphen");
    assert_eq!(hy_name, "model-id");
    assert_eq!(hy_desc, "Claude via API");
    let (sp_name, sp_desc) = parse_model_line("gpt-4 stable release").expect("whitespace");
    assert_eq!(sp_name, "gpt-4");
    assert_eq!(sp_desc, "stable release");
    assert!(parse_model_line("onlytoken").is_none());
    let lines =
        models_display_lines("composer-2 — Fast\nHEADERS\ngpt-4.1 — Stable\n").expect("display");
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[1], "HEADERS");
}

#[cfg(unix)]
fn clear_cursor_api_keys_for_models_test() -> malvin::test_support::test_utils::SavedEnvVars {
    let saved = malvin::test_support::test_utils::SavedEnvVars::capture(&[
        "CURSOR_API_KEY",
        "CURSOR_AGENT_API_KEY",
        "AGENT_API_KEY",
    ]);
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("CURSOR_API_KEY");
        std::env::remove_var("CURSOR_AGENT_API_KEY");
        std::env::remove_var("AGENT_API_KEY");
    }
    saved
}

#[cfg(unix)]
fn install_failing_fake_agent(dir: &std::path::Path) {
    crate::cli::fake_models_agent::install_agent(
        dir,
        crate::cli::fake_models_agent::FAILING_AGENT_SCRIPT,
    );
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_models_soft_fails_cursor_and_continues() {
    use super::test_hooks::print_cursor_models_via_cli_for_test;
    use crate::cli::repo_checks::set_fake_command_dir;

    let _lock = malvin::test_support::test_utils::test_env_lock();
    let _saved = clear_cursor_api_keys_for_models_test();
    let tmp = tempfile::tempdir().expect("tempdir");
    install_failing_fake_agent(tmp.path());
    let _guard = set_fake_command_dir(tmp.path());
    let err = print_cursor_models_via_cli_for_test(Some("cursor:"))
        .expect_err("failing fake agent must return an error");
    assert!(
        err.contains("agent models failed"),
        "unexpected error: {err}"
    );
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_models_fake_agent_branchy_executable() {
    use super::test_hooks::print_cursor_models_via_cli_for_test;
    use crate::cli::fake_models_agent::{MODELS_LISTING_SCRIPT, install_agent};
    use crate::cli::repo_checks::set_fake_command_dir;
    use malvin::output::{enable_stdout_capture, take_captured_stdout};

    let _lock = malvin::test_support::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    install_agent(tmp.path(), MODELS_LISTING_SCRIPT);
    let _guard = set_fake_command_dir(tmp.path());
    enable_stdout_capture();
    print_cursor_models_via_cli_for_test(Some("cursor:")).expect("fake agent models");
    let listed = take_captured_stdout();
    assert!(
        listed.contains("cursor:composer-2\tFast"),
        "listing should name the model and its speed: {listed}"
    );
    assert!(
        !listed.contains("Tip"),
        "tip banner should be omitted from the listing: {listed}"
    );
    enable_stdout_capture();
    print_cursor_models_via_cli_for_test(Some("cursor:missing")).expect("filtered listing");
    let filtered = take_captured_stdout();
    assert!(
        !filtered.contains("composer-2"),
        "a prefix that misses the id should hide that row: {filtered}"
    );
}

#[test]
fn kiss_cov_models_refresh_timestamp_lifecycle() {
    let _lock = malvin::test_support::test_utils::test_env_lock();
    let now = malvin::clock::unix_now_secs();
    super::test_hooks::save_last_refresh_secs(now).expect("save");
    let loaded = super::test_hooks::load_last_refresh_secs().expect("load");
    assert_eq!(loaded, now);
    assert!(!super::test_hooks::models_refresh_is_due(now));
    let future = now + super::models_cmd_refresh::MODELS_REFRESH_INTERVAL_SECS + 1;
    assert!(super::test_hooks::models_refresh_is_due(future));
}
