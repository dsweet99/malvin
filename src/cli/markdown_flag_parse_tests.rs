use crate::cli::Cli;
use clap::Parser;

#[test]
fn global_no_markdown_before_code_subcommand() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "code", "x"]).expect("parse");
    assert!(cli.shared.no_markdown);
}

#[test]
fn global_no_markdown_after_shared_flags_before_kpop() {
    let cli = Cli::try_parse_from([
        "malvin",
        "--model",
        "m",
        "--no-markdown",
        "kpop",
        "x",
    ])
    .expect("parse");
    assert!(cli.shared.no_markdown);
    assert_eq!(cli.shared.model, "m");
}

#[test]
fn do_parses_with_global_no_markdown_without_do_local_flag() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "do", "hi"]).expect("parse");
    assert!(cli.shared.no_markdown);
    match cli.command.as_ref() {
        Some(crate::cli::Commands::Do(d)) => assert_eq!(d.request.as_deref(), Some("hi")),
        _ => panic!("expected Do"),
    }
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
