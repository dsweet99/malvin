use crate::cli::config_defaults::parse_cli_with_config_defaults;
use crate::cli::Cli;
use clap::Parser;
use malvin::config::DEFAULT_MAX_ACP_RETRIES;
use malvin::test_utils::with_isolated_home;

fn parse_ok(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work| {
        out = Some(parse_cli_with_config_defaults(argv).expect("parse").0);
    });
    out.expect("parsed")
}

fn parse_err(argv: &[&str]) -> String {
    let mut out = None;
    with_isolated_home(|_work| {
        out = Some(
            parse_cli_with_config_defaults(argv)
                .expect_err("expected parse error")
                .to_string(),
        );
    });
    out.expect("err")
}

fn cli_accepts_do_and_passes_request() {
    let cli = parse_ok(&["malvin", "--do", "fix the bug"]);
    assert!(cli.do_workflow());
    assert_eq!(cli.first_request().map(String::as_str), Some("fix the bug"));
    assert!(cli.command.is_none());
    assert!(cli.has_do_request());
    assert!(!cli.has_router_request());
}

fn cli_rejects_do_thoughts_flag() {
    let err = Cli::try_parse_from(["malvin", "--do", "--thoughts", "z"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected argument") || msg.contains("--thoughts"),
        "expected --thoughts rejected; got {msg}"
    );
}

fn cli_accepts_all_shared_flags_before_subcommand() {
    let cli = parse_ok(&["malvin", "--model", "cursor:composer-2", "--do", "z"]);
    assert_eq!(cli.shared.model.canonical(), "cursor:composer-2");
    assert!(cli.do_workflow());
    assert_eq!(cli.first_request().map(String::as_str), Some("z"));
}

fn cli_rejects_max_loops_with_pure_do() {
    let msg = parse_err(&["malvin", "--do", "--max-loops", "5", "task"]);
    assert!(
        msg.contains("cannot be used with") || msg.contains("--max-loops"),
        "expected --max-loops conflict rejected; got {msg}"
    );
    let msg2 = parse_err(&["malvin", "--max-loops", "5", "--do", "task"]);
    assert!(
        msg2.contains("cannot be used with") || msg2.contains("--max-loops"),
        "expected --max-loops conflict rejected; got {msg2}"
    );
}

fn cli_rejects_gates_with_pure_do() {
    let msg = parse_err(&["malvin", "--do", "-g", "task"]);
    assert!(
        msg.contains("cannot be used with") || msg.contains("--gates") || msg.contains("-g"),
        "expected --gates conflict rejected; got {msg}"
    );
    let msg2 = parse_err(&["malvin", "--gates", "--do", "task"]);
    assert!(
        msg2.contains("cannot be used with") || msg2.contains("--gates"),
        "expected --gates conflict rejected; got {msg2}"
    );
}

fn cli_rejects_creative_with_pure_do() {
    let msg = parse_err(&["malvin", "--do", "--creative=0.8", "task"]);
    assert!(
        msg.contains("cannot be used with") || msg.contains("--creative"),
        "expected --creative conflict rejected; got {msg}"
    );
}

fn cli_rejects_max_hypotheses_with_pure_do() {
    let msg = parse_err(&["malvin", "--do", "--max-hypotheses", "10", "task"]);
    assert!(
        msg.contains("cannot be used with") || msg.contains("--max-hypotheses"),
        "expected --max-hypotheses conflict rejected; got {msg}"
    );
}

fn cli_rejects_no_kpop_with_pure_do() {
    let msg = parse_err(&["malvin", "--do", "--no-kpop", "task"]);
    assert!(
        msg.contains("cannot be used with") || msg.contains("--no-kpop"),
        "expected --no-kpop conflict rejected; got {msg}"
    );
}

fn cli_rejects_quiet_with_pure_do() {
    let msg = parse_err(&["malvin", "--do", "-q", "task"]);
    assert!(
        msg.contains("cannot be used with") || msg.contains("--quiet") || msg.contains("-q"),
        "expected --quiet conflict rejected; got {msg}"
    );
    let msg2 = parse_err(&["malvin", "--quiet", "--do", "task"]);
    assert!(
        msg2.contains("cannot be used with") || msg2.contains("--quiet"),
        "expected --quiet conflict rejected; got {msg2}"
    );
}

fn cli_accepts_router_flags_when_mixed_with_do() {
    let cli = parse_ok(&["malvin", "-g", "router task", "--do", "do task"]);
    assert!(cli.router.gates);
    assert!(cli.has_router_request());
    assert!(cli.has_do_request());
}

fn cli_accepts_max_acp_retries_global_flag() {
    let cli = parse_ok(&["malvin", "--do", "task"]);
    assert_eq!(cli.shared.max_acp_retries, DEFAULT_MAX_ACP_RETRIES);
    let cli = parse_ok(&["malvin", "--max-acp-retries", "5", "--do", "task"]);
    assert_eq!(cli.shared.max_acp_retries, 5);
}

fn cli_accepts_verbose_short_and_long_global_flags() {
    let cli = parse_ok(&["malvin", "-v", "--do", "x"]);
    assert!(cli.shared.verbose);
    assert!(cli.do_workflow());
    assert_eq!(cli.first_request().map(String::as_str), Some("x"));
    let cli = parse_ok(&["malvin", "--do", "--verbose", "y"]);
    assert!(cli.shared.verbose);
    assert!(cli.do_workflow());
    assert_eq!(cli.first_request().map(String::as_str), Some("y"));
}

#[test]
fn kiss_bundled_cli_do_flow_cli_parse_tests() {
    cli_accepts_do_and_passes_request();
    cli_rejects_do_thoughts_flag();
    cli_accepts_all_shared_flags_before_subcommand();
    cli_rejects_max_loops_with_pure_do();
    cli_rejects_gates_with_pure_do();
    cli_rejects_creative_with_pure_do();
    cli_rejects_no_kpop_with_pure_do();
    cli_rejects_quiet_with_pure_do();
    cli_rejects_max_hypotheses_with_pure_do();
    cli_accepts_router_flags_when_mixed_with_do();
    cli_accepts_max_acp_retries_global_flag();
    cli_accepts_verbose_short_and_long_global_flags();
}
