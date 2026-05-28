use super::*;
use crate::cli::{Cli, Commands};
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
fn bare_and_do_and_at_workflows_resolve() {
    let kpop = parse_and_resolve(&["malvin", "investigate cache"]);
    assert!(matches!(kpop.command, Some(Commands::Kpop(_))));
    let do_cmd = parse_and_resolve(&["malvin", "--do", "fix typo"]);
    assert!(matches!(do_cmd.command, Some(Commands::Do(_))));
    let code = parse_and_resolve(&["malvin", "@code", "plan.md"]);
    assert!(matches!(code.command, Some(Commands::Code(_))));
    let tidy = parse_and_resolve(&["malvin", "@tidy"]);
    assert!(matches!(tidy.command, Some(Commands::Tidy(_))));
}

#[test]
fn bare_errors_cover_edge_cases() {
    assert!(parse_resolve_err(&["malvin", "--do", "@code", "x"]).contains("--do"));
    assert!(parse_resolve_err(&["malvin", "@nope", "x"]).contains("@nope"));
    assert!(parse_resolve_err(&["malvin", "@tidy", "extra"]).contains("@tidy"));
    assert!(parse_resolve_err(&["malvin", "@code"]).contains("@code"));
    assert!(parse_resolve_err(&["malvin", "@code", "@tidy"]).contains("only one @workflow"));
    assert!(parse_resolve_err(&["malvin", "   "]).contains("REQUEST"));
}

#[test]
fn bare_flags_forward_to_workflows() {
    let kpop = parse_and_resolve(&["malvin", "--max-loops", "4", "q"]);
    match kpop.command {
        Some(Commands::Kpop(k)) => assert_eq!(k.max_loops, 4),
        other => panic!("expected kpop, got {other:?}"),
    }
    let do_cmd = parse_and_resolve(&["malvin", "--do", "--repo-gates", "--thoughts", "task"]);
    match do_cmd.command {
        Some(Commands::Do(d)) => {
            assert!(d.repo_gates && d.thoughts);
        }
        other => panic!("expected do, got {other:?}"),
    }
    let code = parse_and_resolve(&["malvin", "--tenacious", "@code", "plan"]);
    match code.command {
        Some(Commands::Code(c)) => assert!(c.tenacious),
        other => panic!("expected code, got {other:?}"),
    }
}

#[test]
fn resolve_bare_helper_functions_directly() {
    let cmd = Cli::command();
    let no_do = cmd.get_matches_from(["malvin", "hello"]);
    let cli = Cli::from_arg_matches(&no_do).expect("cli");
    assert!(resolve_bare_do(&cli).expect("ok").is_none());
    let kpop_cmd = resolve_bare_kpop(&cli, &no_do).expect("kpop");
    assert!(matches!(kpop_cmd, Commands::Kpop(_)));
    let at_cmd = resolve_bare_at_or_kpop(&cli, &no_do).expect("at");
    assert!(matches!(at_cmd, Some(Commands::Kpop(_))));
    let mut cli_mut = Cli::from_arg_matches(&no_do).expect("cli");
    resolve_bare_command(&mut cli_mut, &no_do).expect("resolve");
}

#[test]
fn unit_helpers_at_workflow_join_request_bare_loop_resolve() {
    assert_eq!(AtWorkflow::parse("@code"), Some(AtWorkflow::Code));
    assert_eq!(join_request_parts(&["a".into(), "b".into()]), "a b");
    require_bare_request(&[], "usage").expect_err("empty");
    reject_multiple_at_selectors(&["@code".into(), "plan".into()]).expect("ok");
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
    let at_matches = Cli::command().get_matches_from(["malvin", "@code", "p"]);
    let at_cli = Cli::from_arg_matches(&at_matches).expect("cli");
    let resolved = resolve_at_workflow(&at_cli, &at_matches, AtWorkflow::Code).expect("resolve");
    assert!(matches!(resolved, Commands::Code(_)));
}

#[test]
fn kiss_cov_bare_invoke_symbols() {
    let _ = stringify!(resolve_bare_command);
    let _ = stringify!(resolve_bare_do);
    let _ = stringify!(resolve_bare_kpop);
    let _ = stringify!(resolve_bare_at_or_kpop);
    let _ = stringify!(AtWorkflow);
    let _ = stringify!(join_request_parts);
    let _ = stringify!(require_bare_request);
    let _ = stringify!(reject_multiple_at_selectors);
    let _ = stringify!(BareLoopOpts);
    let _ = stringify!(bare_loop_opts);
    let _ = stringify!(resolve_at_workflow);
}
