use crate::cli::Cli;
use crate::cli::config_defaults::parse_cli_with_config_defaults;
use clap::Parser;
use malvin::config::DEFAULT_MAX_ACP_RETRIES;
use malvin::test_support::test_utils::with_isolated_home;

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

fn cli_rejects_removed_do_watch_and_verbose_flags() {
    for flag in ["--do", "--watch", "--verbose", "-v"] {
        let msg = parse_err(&["malvin", flag, "task"]);
        assert!(
            msg.contains("unexpected argument") || msg.contains(flag),
            "expected {flag} rejected; got {msg}"
        );
    }
}

fn cli_rejects_do_thoughts_flag() {
    let err = Cli::try_parse_from(["malvin", "--thoughts", "z"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected argument") || msg.contains("--thoughts"),
        "expected --thoughts rejected; got {msg}"
    );
}

fn cli_accepts_all_shared_flags_before_subcommand() {
    let cli = parse_ok(&["malvin", "--model", "cursor:composer-2", "z"]);
    assert_eq!(cli.shared.model.first().canonical(), "cursor:composer-2");
    assert_eq!(cli.first_request().map(String::as_str), Some("z"));
}

fn cli_accepts_max_loops_flag() {
    use malvin::config::malvin_config_file::DEFAULT_MAX_LOOPS;
    let cli = parse_ok(&["malvin", "task"]);
    assert_eq!(cli.router.max_loops, DEFAULT_MAX_LOOPS);
    let cli = parse_ok(&["malvin", "--max-loops", "5", "task"]);
    assert_eq!(cli.router.max_loops, 5);
    let msg = parse_err(&["malvin", "--max-loops", "0", "task"]);
    assert!(
        msg.contains("--max-loops") && msg.contains("positive integer"),
        "expected --max-loops 0 rejected; got {msg}"
    );
}

fn cli_rejects_unknown_max_hypotheses_flag() {
    let msg = parse_err(&["malvin", "--max-hypotheses", "10", "task"]);
    assert!(
        msg.contains("unexpected") || msg.contains("unknown") || msg.contains("--max-hypotheses"),
        "expected removed --max-hypotheses flag rejected; got {msg}"
    );
}

fn cli_accepts_max_acp_retries_global_flag() {
    let cli = parse_ok(&["malvin", "task"]);
    assert_eq!(cli.shared.max_acp_retries, DEFAULT_MAX_ACP_RETRIES);
    let cli = parse_ok(&["malvin", "--max-acp-retries", "5", "task"]);
    assert_eq!(cli.shared.max_acp_retries, 5);
}

#[test]
fn kiss_bundled_cli_do_flow_cli_parse_tests() {
    cli_rejects_removed_do_watch_and_verbose_flags();
    cli_rejects_do_thoughts_flag();
    cli_accepts_all_shared_flags_before_subcommand();
    cli_accepts_max_loops_flag();
    cli_rejects_unknown_max_hypotheses_flag();
    cli_accepts_max_acp_retries_global_flag();
}
