use crate::cli::Cli;
use clap::Parser;

#[test]
fn global_no_markdown_before_code_subcommand() {
    let cli =
        Cli::try_parse_from(["malvin", "--no-markdown", "code", "--no-learn", "x"]).expect("parse");
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
        "--no-learn",
        "x",
    ])
    .expect("parse");
    assert!(cli.shared.no_markdown);
    assert_eq!(cli.shared.model, "m");
}

#[test]
fn global_no_markdown_before_bug_subcommand() {
    let cli =
        Cli::try_parse_from(["malvin", "--no-markdown", "bughunt", "--no-learn"]).expect("parse");
    assert!(cli.shared.no_markdown);
    assert!(matches!(cli.command, Some(crate::cli::Commands::Bug(_))));
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
fn tidy_parses_without_request_and_runs_learn() {
    let cli = Cli::try_parse_from(["malvin", "tidy", "--no-learn"]).expect("parse");
    match cli.command {
        Some(crate::cli::Commands::Tidy(tidy)) => assert!(tidy.no_learn),
        _ => panic!("expected tidy"),
    }
}

#[test]
fn models_parses_with_global_no_markdown() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "models"]).expect("parse");
    assert!(cli.shared.no_markdown);
    assert!(matches!(cli.command, Some(crate::cli::Commands::Models(_))));
}

#[test]
fn plan_parses_text_after_plan_path_flag() {
    let cli = Cli::try_parse_from(["malvin", "plan", "--plan_path", "/tmp/p.md", "hello"])
        .expect("parse");
    match cli.command {
        Some(crate::cli::Commands::Plan(p)) => {
            assert_eq!(
                p.plan_path.as_deref(),
                Some(std::path::Path::new("/tmp/p.md"))
            );
            assert_eq!(p.text.as_deref(), Some("hello"));
        }
        _ => panic!("expected plan"),
    }
}

#[test]
fn plan_parses_plan_path_alias_before_text() {
    let cli = Cli::try_parse_from(["malvin", "plan", "--plan-path", "notes/plan.md", "x"])
        .expect("parse");
    match cli.command {
        Some(crate::cli::Commands::Plan(p)) => {
            assert_eq!(
                p.plan_path.as_ref().map(|x| x.as_os_str()),
                Some(std::ffi::OsStr::new("notes/plan.md"))
            );
            assert_eq!(p.text.as_deref(), Some("x"));
        }
        _ => panic!("expected plan"),
    }
}

#[test]
fn plan_parses_without_positional() {
    let cli = Cli::try_parse_from(["malvin", "plan"]).expect("parse");
    match cli.command {
        Some(crate::cli::Commands::Plan(p)) => {
            assert!(p.plan_path.is_none());
            assert!(p.text.is_none());
        }
        _ => panic!("expected plan"),
    }
}
