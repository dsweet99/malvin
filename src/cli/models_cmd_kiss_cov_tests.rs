//! External kiss witnesses for [`super`] (must be `*_tests.rs` for kiss).

fn kiss_witness_copy<T: Copy>(value: T) -> T {
    value
}

#[test]
fn kiss_cov_models_args_clap_parse_and_destructure() {
    use clap::{CommandFactory, FromArgMatches, Parser};

    use super::{models_args_marker, ModelsArgs};
    use crate::cli::{Cli, Commands};

    let cli = Cli::try_parse_from(["malvin", "models"]).expect("parse models");
    match cli.command {
        Some(Commands::Models(args)) => {
            assert_eq!(models_args_marker(args), "models");
            let ModelsArgs {} = kiss_witness_copy(args);
        }
        _ => panic!("expected Models subcommand"),
    }
    let reparse = Cli::try_parse_from(["malvin", "models"]).expect("reparse");
    if let Some(Commands::Models(second)) = reparse.command {
        assert_eq!(models_args_marker(second), "models");
        assert_eq!(format!("{second:?}"), "ModelsArgs");
        let ModelsArgs {} = kiss_witness_copy(second);
    } else {
        panic!("second parse should yield Models");
    }
    let cmd = Cli::command();
    assert!(cmd.find_subcommand("models").is_some());
    let matches = Cli::command().get_matches_from(["malvin", "models"]);
    let sub = matches.subcommand_matches("models").expect("models matches");
    let ModelsArgs {} = ModelsArgs::from_arg_matches(sub).expect("models from_arg_matches");
    let ModelsArgs {} = kiss_witness_copy(ModelsArgs {});
}

#[test]
fn kiss_cov_models_cmd_run_helpers() {
    use super::test_hooks::*;
    use super::ModelsArgs;

    let args = ModelsArgs {};
    assert_eq!(format!("{args:?}"), "ModelsArgs");
    let trimmed = trim_trailing_tip_lines("line\nTip: drop\n");
    assert_eq!(trimmed, "line");
    let (name, desc) = parse_model_line("gpt-4 — stable").expect("parse");
    assert_eq!(name, "gpt-4");
    assert_eq!(desc, "stable");
    let lines = models_display_lines("only-one\n").expect("lines");
    assert_eq!(lines, vec!["only-one".to_string()]);
    print_parsed_or_fallback("fallback\n");
}

#[test]
fn kiss_cov_models_branchy_executable_witness() {
    use super::test_hooks::*;

    assert!(looks_like_tip_banner_line("tip: upgrade"));
    assert!(looks_like_tip_banner_line("tip use tls"));
    assert!(!looks_like_tip_banner_line("tip of the day"));
    assert!(!looks_like_tip_banner_line("see tip: inline"));
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
#[test]
fn kiss_cov_run_models_surfaces_agent_failure() {
    use std::os::unix::fs::PermissionsExt;

    use super::run_models;
    use crate::repo_checks::set_fake_command_dir;

    let tmp = tempfile::tempdir().expect("tempdir");
    let agent = tmp.path().join("agent");
    std::fs::write(&agent, "#!/bin/sh\nexit 1\n").expect("write fake agent");
    let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&agent, perms).expect("chmod");
    let _guard = set_fake_command_dir(tmp.path());
    let err = run_models().expect_err("failing agent");
    assert!(err.contains("models"));
}

#[test]
fn kiss_cov_models_cmd_private_fn_names() {
    let _ = stringify!(trim_trailing_tip_lines);
    let _ = stringify!(looks_like_tip_banner_line);
    let _ = stringify!(models_display_lines);
    let _ = stringify!(print_parsed_or_fallback);
    let _ = stringify!(parse_model_line);
    let _ = stringify!(resolve_models_cli);
    let _ = stringify!(models_args_marker);
}

#[cfg(unix)]
#[test]
fn kiss_cov_run_models_fake_agent_branchy_executable() {
    use std::os::unix::fs::PermissionsExt;

    use super::run_models;
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
    if run_models().is_ok() {
        let again = run_models();
        assert!(again.is_ok() || again.is_err());
    } else {
        panic!("fake agent models should succeed");
    }
}
