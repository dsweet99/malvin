use crate::cli::Cli;
use clap::Parser;

#[test]
fn global_no_markdown_before_code_subcommand() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "code", "x"]).expect("parse");
    assert!(cli.shared.no_markdown);
}

#[test]
fn global_no_markdown_after_shared_flags_before_inspire() {
    let cli = Cli::try_parse_from([
        "malvin",
        "--model",
        "m",
        "--no-markdown",
        "inspire",
        "x",
    ])
    .expect("parse");
    assert!(cli.shared.no_markdown);
    assert_eq!(cli.shared.model, "m");
}

#[test]
fn do_parses_with_global_no_markdown_without_do_local_flag() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "--do", "hi"]).expect("parse");
    assert!(cli.shared.no_markdown);
    assert!(cli.do_workflow);
    assert_eq!(cli.request.as_deref(), Some("hi"));
}

#[test]
fn tidy_parses_with_global_no_markdown_and_without_request() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "tidy"]).expect("parse");
    assert!(cli.shared.no_markdown);
    assert!(matches!(cli.command, Some(crate::cli::Commands::Tidy(_))));
}

#[test]
fn models_parses_with_global_no_markdown() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "models"]).expect("parse");
    assert!(cli.shared.no_markdown);
    assert!(matches!(cli.command, Some(crate::cli::Commands::Models(_))));
}

#[test]
fn global_quiet_long_and_short_parse() {
    let long = Cli::try_parse_from(["malvin", "--quiet", "hello"]).expect("parse");
    assert!(long.shared.quiet);
    assert_eq!(long.request.as_deref(), Some("hello"));
    let short = Cli::try_parse_from(["malvin", "-q", "hello"]).expect("parse");
    assert!(short.shared.quiet);
}

#[test]
fn quiet_parses_on_router_wrappers() {
    for argv in [
        ["malvin", "-q", "tidy"].as_slice(),
        ["malvin", "--quiet", "delight"].as_slice(),
        ["malvin", "-q", "explain", "topic"].as_slice(),
    ] {
        let cli = Cli::try_parse_from(argv).expect("parse");
        assert!(cli.shared.quiet, "argv={argv:?}");
    }
}

#[test]
fn tidy_short_q_is_global_quiet_not_deprecated_quick() {
    let cli = Cli::try_parse_from(["malvin", "tidy", "-q"]).expect("parse");
    assert!(cli.shared.quiet);
    match cli.command {
        Some(crate::cli::Commands::Tidy(t)) => assert!(!t.quick),
        other => panic!("expected Tidy, got {other:?}"),
    }
}
