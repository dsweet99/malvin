use super::models_cmd::test_hooks::*;

fn trim_trailing_tips_drops_banner() {
    let t = "a\nb\nTip: upgrade\n";
    assert_eq!(trim_trailing_tip_lines(t).lines().count(), 2);
}

fn trim_trailing_tips_drops_tip_space_banner_without_colon() {
    let t = "a\nb\ntip use TLS in prod\n";
    assert_eq!(trim_trailing_tip_lines(t).lines().count(), 2);
}

fn trim_trailing_tips_keeps_last_line_that_mentions_tip_mid_sentence() {
    let t = "composer-2 — Fast\nSee tip: use TLS in prod\n";
    assert_eq!(
        trim_trailing_tip_lines(t),
        "composer-2 — Fast\nSee tip: use TLS in prod"
    );
}

fn trim_trailing_tips_keeps_line_starting_with_tip_of_english_phrase() {
    let t = "composer-2 — Fast\nTip of the iceberg — latency matters\n";
    assert_eq!(
        trim_trailing_tip_lines(t),
        "composer-2 — Fast\nTip of the iceberg — latency matters"
    );
}

fn parse_model_line_splits_em_dash() {
    let (n, d) = parse_model_line("composer-2 — Fast").expect("parse");
    assert_eq!(n, "composer-2");
    assert_eq!(d, "Fast");
}

fn parse_model_line_splits_ascii_hyphen_when_name_has_many_words() {
    let line = "my production inference tier one model id - Claude via API";
    let (n, d) = parse_model_line(line).expect("parse");
    assert_eq!(n, "my production inference tier one model id");
    assert_eq!(d, "Claude via API");
}

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

fn models_subcommand_parse_invokes_cli_helpers() {
    use crate::cli::{AdminArgs, AdminCommand, Cli, Commands};
    use clap::Parser;
    let cli = Cli::try_parse_from(["malvin", "admin", "models"]).expect("parse");
    assert!(matches!(
        cli.command,
        Some(Commands::Admin(AdminArgs {
            command: Some(AdminCommand::Models(_)),
        }))
    ));
    let refresh =
        Cli::try_parse_from(["malvin", "admin", "models", "--refresh", "pi:"]).expect("parse");
    match refresh.command {
        Some(Commands::Admin(AdminArgs {
            command: Some(AdminCommand::Models(args)),
        })) => assert!(args.refresh),
        _ => panic!("expected Admin::Models"),
    }
}

#[cfg(unix)]
fn write_fake_models_agent(dir: &std::path::Path) -> std::path::PathBuf {
    super::fake_models_agent::install_agent(dir, super::fake_models_agent::MODELS_LISTING_SCRIPT)
}

#[cfg(unix)]
fn run_models_reads_fake_agent_models_output() {
    use crate::cli::repo_checks::set_fake_command_dir;

    let tmp = tempfile::tempdir().expect("tempdir");
    let agent = write_fake_models_agent(tmp.path());
    let _guard = set_fake_command_dir(tmp.path());
    malvin::output::enable_stdout_capture();
    print_cursor_models_via_cli_for_test(Some("cursor:")).expect("fake agent models");
    let out = malvin::output::take_captured_stdout();
    assert!(out.contains("cursor:composer-2\tFast"), "{out}");
    assert!(!out.contains("Tip"), "{out}");
    let path = resolve_models_cli().expect("fake agent on fake PATH");
    assert_eq!(path, agent);
}

fn model_list_suffix_places_config_aliases_above_current() {
    use super::models_cmd::print_models_suffix;
    use malvin::output::{enable_stdout_capture, take_captured_stdout};
    use malvin::test_support::test_utils::with_isolated_home;
    use malvin::workspace::workspace_paths::malvin_config_path;

    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "[agent]\nmodel = \"cursor:auto\"\n").expect("write");
        enable_stdout_capture();
        print_models_suffix("cursor:auto");
        let empty = take_captured_stdout();
        assert!(!empty.contains("[aliases.models]"), "{empty}");
        assert!(empty.contains("Current: cursor:auto"), "{empty}");
        std::fs::write(
            &path,
            "[aliases.models]\nnova = \"pi:openai/gpt-5\"\nastra = \"cursor:gpt-5\"\n[agent]\nmodel = \"cursor:auto\"\n",
        )
        .expect("write");
        enable_stdout_capture();
        print_models_suffix("cursor:auto");
        let out = take_captured_stdout();
        let header = out.find("[aliases.models]").expect("header");
        let astra = out.find("astra\tcursor:gpt-5").expect("astra");
        let nova = out.find("nova\tpi:openai/gpt-5").expect("nova");
        let current = out.find("Current: cursor:auto").expect("current");
        assert!(header < astra && astra < nova && nova < current, "{out}");
    });
}

fn current_model_label_reads_config_or_default() {
    use super::models_cmd::test_hooks::{current_model_label, print_current_footer};
    use malvin::output::{enable_stdout_capture, take_captured_stdout};

    malvin::test_support::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        malvin::config::malvin_config_file::open_malvin_config(work).expect("seed");
        let label = current_model_label();
        assert!(label.starts_with("cursor:"), "{label}");
        enable_stdout_capture();
        print_current_footer();
        let out = take_captured_stdout();
        assert!(out.contains("Current:"), "{out}");
        std::env::set_current_dir(cwd).expect("restore");
    });
}

fn cursor_list_models_timeout_honors_env() {
    use super::models_cmd::test_hooks::cursor_list_models_timeout;
    use malvin::test_support::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let prior = std::env::var_os("MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS", "123");
    }
    assert_eq!(
        cursor_list_models_timeout(),
        std::time::Duration::from_millis(123)
    );
    #[allow(unsafe_code)]
    unsafe {
        match prior {
            Some(v) => std::env::set_var("MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS", v),
            None => std::env::remove_var("MALVIN_CURSOR_LIST_MODELS_TIMEOUT_MS"),
        }
    }
}

fn sdk_catalog_empty_is_detected_even_when_auto_would_be_injected() {
    assert!(!sdk_catalog_has_model_rows(""));
    assert!(!sdk_catalog_has_model_rows("\n\n"));
    assert!(sdk_catalog_has_model_rows("cursor:composer-2\tFast\n"));
    let injected = sdk_model_rows_from_stdout("");
    assert_eq!(injected, vec!["cursor:auto".to_string()]);
}

#[test]
fn kiss_bundled_cli_models_cmd_tests() {
    trim_trailing_tips_drops_banner();
    trim_trailing_tips_drops_tip_space_banner_without_colon();
    trim_trailing_tips_keeps_last_line_that_mentions_tip_mid_sentence();
    trim_trailing_tips_keeps_line_starting_with_tip_of_english_phrase();
    parse_model_line_splits_em_dash();
    parse_model_line_splits_ascii_hyphen_when_name_has_many_words();
    models_display_lines_keeps_unparsed_single_token_between_parsed_rows();
    models_subcommand_parse_invokes_cli_helpers();
    run_models_reads_fake_agent_models_output();
    current_model_label_reads_config_or_default();
    model_list_suffix_places_config_aliases_above_current();
    cursor_list_models_timeout_honors_env();
    sdk_catalog_empty_is_detected_even_when_auto_would_be_injected();
}
