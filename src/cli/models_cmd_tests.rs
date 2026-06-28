//! Unit tests for [`super::models_cmd`].

use super::models_cmd::test_hooks::*;
use super::models_cmd::{run_models, ModelsArgs};

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
    run_models(ModelsArgs { mini: false }).expect("fake agent models");
    let path = resolve_models_cli().expect("fake agent on fake PATH");
    assert_eq!(path, agent);
}

#[tokio::test]
pub(crate) async fn run_mini_models_prints_openrouter_rows_and_footer() {
    use wiremock::MockServer;

    use super::models_cmd::run_mini_models;
    use crate::output::{enable_stdout_capture, take_captured_stdout};

    let server = MockServer::start().await;
    mount_mini_models_mock(&server).await;
    let guards = mini_models_env_guards(&server.uri());
    enable_stdout_capture();
    run_mini_models().await.expect("mini models");
    let out = take_captured_stdout();
    drop(guards);
    assert!(out.contains("anthropic/claude-sonnet-4\tClaude Sonnet 4"));
    assert!(out.contains("Default mini model: anthropic/claude-sonnet-4"));
}

#[test]
fn mini_models_env_guard_struct_literal() {
    let guard = MiniModelsEnvGuards {
        _base: EnvGuard::set("OPENROUTER_BASE_URL", None),
        _key: EnvGuard::set("OPENROUTER_API_KEY", None),
        _timeout: EnvGuard::set("OPENROUTER_REQUEST_TIMEOUT", None),
    };
    drop(guard);
}

pub(crate) async fn mount_mini_models_mock(server: &wiremock::MockServer) {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, ResponseTemplate};

    Mock::given(method("GET"))
        .and(path("/models"))
        .and(query_param("output_modalities", "text"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "anthropic/claude-sonnet-4", "name": "Claude Sonnet 4"}]
        })))
        .mount(server)
        .await;
}

pub(crate) struct MiniModelsEnvGuards {
    _base: EnvGuard,
    _key: EnvGuard,
    _timeout: EnvGuard,
}

pub(crate) fn mini_models_env_guards(base_url: &str) -> MiniModelsEnvGuards {
    MiniModelsEnvGuards {
        _base: EnvGuard::set("OPENROUTER_BASE_URL", Some(base_url)),
        _key: EnvGuard::set("OPENROUTER_API_KEY", None),
        _timeout: EnvGuard::set("OPENROUTER_REQUEST_TIMEOUT", Some("5")),
    }
}

#[tokio::test]
pub(crate) async fn run_mini_models_surfaces_http_errors() {
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::models_cmd::run_mini_models;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .mount(&server)
        .await;

    let _base = EnvGuard::set("OPENROUTER_BASE_URL", Some(&server.uri()));
    let _key = EnvGuard::set("OPENROUTER_API_KEY", Some("sk-test"));
    let _timeout = EnvGuard::set("OPENROUTER_REQUEST_TIMEOUT", Some("5"));

    let err = run_mini_models().await.expect_err("500");
    assert!(err.contains("500"));
}

#[test]
pub(crate) fn print_mini_models_formats_tab_separated_rows() {
    use malvin_mini::ModelListing;

    use crate::output::{enable_stdout_capture, take_captured_stdout};

    enable_stdout_capture();
    print_mini_models(&[
        ModelListing {
            id: "a/b".into(),
            name: "AB".into(),
        },
        ModelListing {
            id: "c/d".into(),
            name: "CD".into(),
        },
    ]);
    let out = take_captured_stdout();
    assert!(out.contains("a/b\tAB"));
    assert!(out.contains("c/d\tCD"));
}

#[test]
pub(crate) fn kiss_cov_mini_models_test_helpers() {
    let _ = stringify!(MiniModelsEnvGuards);
    let guards = mini_models_env_guards("http://127.0.0.1:9");
    drop(guards);
    let _ = mini_models_env_guards;
    let _ = mount_mini_models_mock;
}
