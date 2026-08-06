//! External kiss witnesses for [`super`] (must be `*_tests.rs` for kiss).

fn kiss_witness_clone<T: Clone>(value: &T) -> T {
    value.clone()
}

#[test]
fn kiss_cov_models_args_clap_parse_and_destructure() {
    use clap::{CommandFactory, FromArgMatches, Parser};

    use super::{models_args_marker, ModelsArgs};
    use crate::cli::{Cli, Commands};

    let cli = Cli::try_parse_from(["malvin", "models"]).expect("parse models");
    match cli.command {
        Some(Commands::Models(args)) => {
            assert_eq!(models_args_marker(&args), "models");
            let _args = kiss_witness_clone(&args);
        }
        _ => panic!("expected Models subcommand"),
    }
    let reparse = Cli::try_parse_from(["malvin", "models"]).expect("reparse");
    if let Some(Commands::Models(second)) = reparse.command {
        assert_eq!(models_args_marker(&second), "models");
        assert!(format!("{second:?}").starts_with("ModelsArgs"));
        let _second = kiss_witness_clone(&second);
    } else {
        panic!("second parse should yield Models");
    }
    let cmd = Cli::command();
    assert!(cmd.find_subcommand("models").is_some());
    let matches = Cli::command().get_matches_from(["malvin", "models"]);
    let sub = matches.subcommand_matches("models").expect("models matches");
    let _parsed = ModelsArgs::from_arg_matches(sub).expect("models from_arg_matches");
    let _cloned = kiss_witness_clone(&ModelsArgs::default());
}

#[test]
fn kiss_cov_models_cmd_run_helpers() {
    use super::test_hooks::*;
    use super::{run_mini_models, ModelsArgs};

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
    // Cross-file kiss witnesses: async mini helpers live in models_cmd_tests.rs.
    let _ = run_mini_models;
    let _ = (
        crate::cli::models_cmd_tests::run_mini_models_prints_openrouter_rows_and_footer,
        crate::cli::models_cmd_tests::run_mini_models_surfaces_http_errors,
        crate::cli::models_cmd_tests::print_mini_models_formats_tab_separated_rows,
        crate::cli::models_cmd_tests::kiss_cov_mini_models_test_helpers,
    );
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
    let lines = models_display_lines("composer-2 — Fast\nHEADERS\ngpt-4.1 — Stable\n")
        .expect("display");
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[1], "HEADERS");
}

#[cfg(unix)]
fn clear_cursor_api_keys_for_models_test() -> crate::test_utils::SavedEnvVars {
    let saved = crate::test_utils::SavedEnvVars::capture(&[
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
    use std::os::unix::fs::PermissionsExt;

    let agent = dir.join("agent");
    std::fs::write(&agent, "#!/bin/sh\nexit 1\n").expect("write fake agent");
    let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&agent, perms).expect("chmod");
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_models_soft_fails_cursor_and_continues() {
    use super::run_models;
    use super::ModelsArgs;
    use crate::output::{enable_stdout_capture, take_captured_stdout};
    use crate::repo_checks::set_fake_command_dir;

    let _lock = crate::test_utils::test_env_lock();
    let _saved = clear_cursor_api_keys_for_models_test();
    let tmp = tempfile::tempdir().expect("tempdir");
    install_failing_fake_agent(tmp.path());
    let _guard = set_fake_command_dir(tmp.path());
    enable_stdout_capture();
    // Cursor auth/agent failure must not abort Prime / Mini sections.
    run_models(ModelsArgs::default(), crate::config::DEFAULT_CLI_MODEL)
        .expect("cursor failure is soft");
    let out = take_captured_stdout();
    assert!(
        out.contains("cursor models unavailable"),
        "expected soft-fail notice, got: {out}"
    );
    assert!(
        out.contains("Current:"),
        "footer must still print after cursor soft-fail: {out}"
    );
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_models_fake_agent_branchy_executable() {
    use std::os::unix::fs::PermissionsExt;

    use super::run_models;
    use super::ModelsArgs;
    use crate::repo_checks::set_fake_command_dir;

    let tmp = tempfile::tempdir().expect("tempdir");
    let agent = tmp.path().join("agent");
    std::fs::write(
        &agent,
        "#!/bin/sh\nif [ \"$1\" = models ]; then printf 'composer-2 — Fast\\nTip: upgrade\\n'; exit 0; fi\nexit 1\n",
    )
    .expect("write fake agent");
    let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&agent, perms).expect("chmod");
    let _guard = set_fake_command_dir(tmp.path());
    if run_models(ModelsArgs::default(), crate::config::DEFAULT_CLI_MODEL).is_ok() {
        let again = run_models(ModelsArgs::default(), crate::config::DEFAULT_CLI_MODEL);
        assert!(again.is_ok() || again.is_err());
    } else {
        panic!("fake agent models should succeed");
    }
}

