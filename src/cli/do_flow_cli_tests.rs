use clap::Parser;

use crate::cli::Cli;
use malvin::config::DEFAULT_MAX_ACP_RETRIES;

fn cli_accepts_do_and_passes_request() {
    let cli = Cli::try_parse_from(["malvin", "--do", "fix the bug"]).expect("parse");
    assert!(cli.do_workflow);
    assert_eq!(cli.first_request().map(String::as_str), Some("fix the bug"));
    assert!(cli.command.is_none());
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
    let cli = Cli::try_parse_from(["malvin", "--model", "cursor:composer-2", "--do", "z"])
        .expect("parse");
    assert_eq!(cli.shared.model.canonical(), "cursor:composer-2");
    assert!(cli.do_workflow);
    assert_eq!(cli.first_request().map(String::as_str), Some("z"));
}

fn cli_rejects_max_loops_with_do() {
    let err =
        Cli::try_parse_from(["malvin", "--do", "--max-loops", "5", "task"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--max-loops"),
        "expected --max-loops conflict rejected; got {msg}"
    );

    let err2 =
        Cli::try_parse_from(["malvin", "--max-loops", "5", "--do", "task"]).expect_err("parse");
    let msg2 = err2.to_string();
    assert!(
        msg2.contains("cannot be used with") || msg2.contains("--max-loops"),
        "expected --max-loops conflict rejected; got {msg2}"
    );
}

fn cli_rejects_gates_with_do() {
    let err = Cli::try_parse_from(["malvin", "--do", "-g", "task"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--gates") || msg.contains("-g"),
        "expected --gates conflict rejected; got {msg}"
    );

    let err2 = Cli::try_parse_from(["malvin", "--gates", "--do", "task"]).expect_err("parse");
    let msg2 = err2.to_string();
    assert!(
        msg2.contains("cannot be used with") || msg2.contains("--gates"),
        "expected --gates conflict rejected; got {msg2}"
    );
}

fn cli_rejects_creative_with_do() {
    let err = Cli::try_parse_from(["malvin", "--do", "--creative=0.8", "task"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--creative"),
        "expected --creative conflict rejected; got {msg}"
    );
}

fn cli_rejects_max_hypotheses_with_do() {
    let err = Cli::try_parse_from(["malvin", "--do", "--max-hypotheses", "10", "task"])
        .expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--max-hypotheses"),
        "expected --max-hypotheses conflict rejected; got {msg}"
    );
}

fn cli_rejects_no_kpop_with_do() {
    let err = Cli::try_parse_from(["malvin", "--do", "--no-kpop", "task"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--no-kpop"),
        "expected --no-kpop conflict rejected; got {msg}"
    );
}

fn cli_rejects_quiet_with_do() {
    let err = Cli::try_parse_from(["malvin", "--do", "-q", "task"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--quiet") || msg.contains("-q"),
        "expected --quiet conflict rejected; got {msg}"
    );

    let err2 = Cli::try_parse_from(["malvin", "--quiet", "--do", "task"]).expect_err("parse");
    let msg2 = err2.to_string();
    assert!(
        msg2.contains("cannot be used with") || msg2.contains("--quiet"),
        "expected --quiet conflict rejected; got {msg2}"
    );
}

fn cli_accepts_max_acp_retries_global_flag() {
    let cli = Cli::try_parse_from(["malvin", "--do", "task"]).expect("parse");
    assert_eq!(cli.shared.max_acp_retries, DEFAULT_MAX_ACP_RETRIES);

    let cli =
        Cli::try_parse_from(["malvin", "--max-acp-retries", "5", "--do", "task"]).expect("parse");
    assert_eq!(cli.shared.max_acp_retries, 5);
}

fn cli_accepts_verbose_short_and_long_global_flags() {
    let cli = Cli::try_parse_from(["malvin", "-v", "--do", "x"]).expect("parse");
    assert!(cli.shared.verbose);
    assert!(cli.do_workflow);
    assert_eq!(cli.first_request().map(String::as_str), Some("x"));

    let cli = Cli::try_parse_from(["malvin", "--do", "--verbose", "y"]).expect("parse");
    assert!(cli.shared.verbose);
    assert!(cli.do_workflow);
    assert_eq!(cli.first_request().map(String::as_str), Some("y"));
}

#[test]
fn kiss_bundled_cli_do_flow_cli_parse_tests() {
    cli_accepts_do_and_passes_request();
    cli_rejects_do_thoughts_flag();
    cli_accepts_all_shared_flags_before_subcommand();
    cli_rejects_max_loops_with_do();
    cli_rejects_gates_with_do();
    cli_rejects_creative_with_do();
    cli_rejects_no_kpop_with_do();
    cli_rejects_quiet_with_do();
    cli_rejects_max_hypotheses_with_do();
    cli_accepts_max_acp_retries_global_flag();
    cli_accepts_verbose_short_and_long_global_flags();
}
