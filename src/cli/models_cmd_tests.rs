//! Unit tests for [`super::models_cmd`].

use super::models_cmd::test_hooks::*;
use super::models_cmd::run_models;

#[test]
fn trim_trailing_tips_drops_banner() {
    let t = "a\nb\nTip: upgrade\n";
    assert_eq!(trim_trailing_tip_lines(t).lines().count(), 2);
}

#[test]
fn trim_trailing_tips_drops_tip_space_banner_without_colon() {
    let t = "a\nb\ntip use TLS in prod\n";
    assert_eq!(trim_trailing_tip_lines(t).lines().count(), 2);
}

#[test]
fn trim_trailing_tips_keeps_last_line_that_mentions_tip_mid_sentence() {
    let t = "composer-2 — Fast\nSee tip: use TLS in prod\n";
    assert_eq!(
        trim_trailing_tip_lines(t),
        "composer-2 — Fast\nSee tip: use TLS in prod"
    );
}

#[test]
fn trim_trailing_tips_keeps_line_starting_with_tip_of_english_phrase() {
    let t = "composer-2 — Fast\nTip of the iceberg — latency matters\n";
    assert_eq!(
        trim_trailing_tip_lines(t),
        "composer-2 — Fast\nTip of the iceberg — latency matters"
    );
}

#[test]
fn parse_model_line_splits_em_dash() {
    let (n, d) = parse_model_line("composer-2 — Fast").expect("parse");
    assert_eq!(n, "composer-2");
    assert_eq!(d, "Fast");
}

#[test]
fn parse_model_line_splits_ascii_hyphen_when_name_has_many_words() {
    let line = "my production inference tier one model id - Claude via API";
    let (n, d) = parse_model_line(line).expect("parse");
    assert_eq!(n, "my production inference tier one model id");
    assert_eq!(d, "Claude via API");
}

#[test]
fn models_display_lines_keeps_unparsed_single_token_between_parsed_rows() {
    let text = "composer-2 — Fast\nHEADERS\ngpt-4.1 — Stable";
    let lines = models_display_lines(text).expect("non-empty");
    assert_eq!(
        lines,
        vec![
            "composer-2\tFast".to_string(),
            "HEADERS".to_string(),
            "gpt-4.1\tStable".to_string(),
        ]
    );
}

#[test]
fn models_subcommand_parse_invokes_cli_helpers() {
    use crate::cli::{Cli, Commands};
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "models"]).expect("parse");
    assert!(matches!(cli.command, Some(Commands::Models(_))));
}

#[cfg(unix)]
#[test]
fn run_models_reads_fake_agent_models_output() {
    use std::os::unix::fs::PermissionsExt;

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
    std::fs::set_permissions(&agent, perms).expect("chmod fake agent");
    let _guard = set_fake_command_dir(tmp.path());
    run_models().expect("fake agent models");
    let path = resolve_models_cli().expect("fake agent on fake PATH");
    assert_eq!(path, agent);
}
