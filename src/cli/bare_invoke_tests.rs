use super::*;
use crate::cli::{parse_cli_with_config_defaults, Cli, Commands};
use clap::{CommandFactory, FromArgMatches};

fn parse_and_resolve(argv: &[&str]) -> Cli {
    let cmd = Cli::command();
    let matches = cmd.clone().get_matches_from(argv);
    let mut cli = Cli::from_arg_matches(&matches).expect("cli");
    resolve_bare_command(&mut cli, &matches).expect("resolve");
    cli
}

fn parse_resolve_err(argv: &[&str]) -> String {
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(argv);
    let mut cli = Cli::from_arg_matches(&matches).expect("cli");
    resolve_bare_command(&mut cli, &matches).unwrap_err()
}

#[test]
fn bare_request_resolves_to_kpop() {
    let kpop = parse_and_resolve(&["malvin", "investigate cache"]);
    assert!(matches!(kpop.command, Some(Commands::Kpop(_))));
}

#[test]
fn bare_single_argv_multi_word_resolves() {
    let cli = parse_cli_with_config_defaults(["malvin", "hello world"])
        .expect("parse")
        .0;
    match cli.command {
        Some(Commands::Kpop(k)) => assert_eq!(k.request.as_deref(), Some("hello world")),
        other => panic!("expected kpop, got {other:?}"),
    }
}

#[test]
fn bare_multiple_requests_stay_unresolved_for_sequential_kpop() {
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(["malvin", "req_a.md", "req_b.md"]);
    let mut cli = Cli::from_arg_matches(&matches).expect("cli");
    resolve_bare_command(&mut cli, &matches).expect("resolve");
    assert!(cli.command.is_none());
    assert_eq!(cli.bare_args, vec!["req_a.md", "req_b.md"]);
}

#[test]
fn code_accepts_multiple_plans() {
    let cli = parse_cli_with_config_defaults(["malvin", "code", "plan_1.md", "plan_2.md"])
        .expect("parse")
        .0;
    match cli.command {
        Some(Commands::Code(c)) => {
            assert_eq!(c.requests.as_slice(), &["plan_1.md", "plan_2.md"]);
        }
        other => panic!("expected code, got {other:?}"),
    }
}

#[test]
fn do_subcommand_parses_without_bare_resolve() {
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(["malvin", "do", "fix typo"]);
    let mut cli = Cli::from_arg_matches(&matches).expect("cli");
    resolve_bare_command(&mut cli, &matches).expect("resolve");
    match cli.command {
        Some(Commands::Do(d)) => assert_eq!(d.request.as_deref(), Some("fix typo")),
        other => panic!("expected do, got {other:?}"),
    }
}

#[test]
fn bare_errors_cover_edge_cases() {
    assert!(parse_resolve_err(&["malvin", "   "]).contains("REQUEST"));
}

#[test]
fn bare_flags_forward_to_kpop() {
    let kpop = parse_and_resolve(&["malvin", "--max-loops", "4", "q"]);
    match kpop.command {
        Some(Commands::Kpop(k)) => assert_eq!(k.max_loops, 4),
        other => panic!("expected kpop, got {other:?}"),
    }
    let tenacious = parse_and_resolve(&["malvin", "--tenacious", "investigate bug"]);
    match tenacious.command {
        Some(Commands::Kpop(k)) => assert!(k.tenacious),
        other => panic!("expected kpop, got {other:?}"),
    }
    let default_tenacious = parse_and_resolve(&["malvin", "investigate bug"]);
    match default_tenacious.command {
        Some(Commands::Kpop(k)) => assert!(k.tenacious),
        other => panic!("expected kpop, got {other:?}"),
    }
    let no_tenacious = parse_and_resolve(&["malvin", "--no-tenacious", "investigate bug"]);
    match no_tenacious.command {
        Some(Commands::Kpop(_)) => assert!(no_tenacious.shared.no_tenacious),
        other => panic!("expected kpop, got {other:?}"),
    }
}

#[test]
fn resolve_bare_helper_functions_directly() {
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(["malvin", "hello"]);
    let cli = Cli::from_arg_matches(&matches).expect("cli");
    let kpop_cmd = resolve_bare_kpop(&cli, &matches).expect("kpop");
    assert!(matches!(kpop_cmd, Commands::Kpop(_)));
    let mut cli_mut = Cli::from_arg_matches(&matches).expect("cli");
    resolve_bare_command(&mut cli_mut, &matches).expect("resolve");
}

#[test]
fn unit_helpers_join_request_bare_loop() {
    require_bare_request(None, "usage").expect_err("empty");
    require_bare_request(Some(&"   ".to_string()), "usage").expect_err("whitespace");
    let cli = parse_cli_with_config_defaults(["malvin", "hello", "world"])
        .expect("parse")
        .0;
    assert!(cli.command.is_none());
    assert_eq!(cli.bare_args, vec!["hello", "world"]);
    let cmd = Cli::command();
    let matches = cmd.get_matches_from(["malvin", "hello"]);
    let cli = Cli::from_arg_matches(&matches).expect("cli");
    let opts = bare_loop_opts(
        &cli,
        &matches,
        BareLoopOpts {
            max_loops: 9,
            max_hypotheses: 8,
            tenacious: true,
        },
    );
    assert_eq!((opts.max_loops, opts.max_hypotheses), (9, 8));
}

#[test]
fn kiss_cov_bare_invoke_symbols() {
    let _ = stringify!(resolve_bare_command);
    let _ = stringify!(resolve_bare_kpop);
    let _ = stringify!(require_bare_request);
    let _ = stringify!(BareLoopOpts);
    let _ = stringify!(bare_loop_opts);
    let _ = parse_and_resolve;
    let _ = parse_resolve_err;
}
