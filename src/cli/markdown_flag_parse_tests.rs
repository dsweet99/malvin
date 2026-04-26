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
fn do_parses_with_global_no_markdown_without_do_local_flag() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "do", "hi"]).expect("parse");
    assert!(cli.shared.no_markdown);
    match &cli.command {
        crate::cli::Commands::Do(d) => assert_eq!(d.request, "hi"),
        _ => panic!("expected Do"),
    }
}

#[test]
fn tidy_parses_with_global_no_markdown_and_without_request() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "tidy"]).expect("parse");
    assert!(cli.shared.no_markdown);
    assert!(matches!(cli.command, crate::cli::Commands::Tidy(_)));
}

#[test]
fn tidy_parses_without_request_and_runs_learn() {
    let cli = Cli::try_parse_from(["malvin", "tidy", "--no-learn"]).expect("parse");
    match cli.command {
        crate::cli::Commands::Tidy(tidy) => assert!(tidy.no_learn),
        _ => panic!("expected tidy"),
    }
}

#[test]
fn sync_parses_with_global_no_markdown_and_request() {
    let cli = Cli::try_parse_from(["malvin", "--no-markdown", "sync", "--no-learn", "x"])
        .expect("parse");
    assert!(cli.shared.no_markdown);
    match cli.command {
        crate::cli::Commands::Sync {
            request,
            no_learn,
            max_loops,
            ..
        } => {
            assert_eq!(request, "x");
            assert!(no_learn);
            assert_eq!(max_loops, 5);
        }
        _ => panic!("expected Sync"),
    }
}

#[test]
fn schedule_parses_with_workers_and_job_file() {
    let cli = Cli::try_parse_from([
        "malvin",
        "schedule",
        "--workers",
        "2",
        "/tmp/jobs.json",
    ])
    .expect("parse");
    match cli.command {
        crate::cli::Commands::Schedule(schedule) => {
            assert_eq!(schedule.workers, 2);
            assert_eq!(schedule.jobs_path, "/tmp/jobs.json");
        }
        _ => panic!("expected Schedule"),
    }
}

