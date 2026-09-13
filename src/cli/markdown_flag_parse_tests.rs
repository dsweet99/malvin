use crate::cli::Cli;
use clap::Parser;

#[test]
fn global_quiet_long_and_short_parse() {
    let long = Cli::try_parse_from(["malvin", "--quiet", "hello"]).expect("parse");
    assert!(long.router.quiet);
    assert_eq!(long.request.as_deref(), Some("hello"));
    let short = Cli::try_parse_from(["malvin", "-q", "hello"]).expect("parse");
    assert!(short.router.quiet);
}

#[test]
fn quiet_parses_on_router_wrappers() {
    let cli = Cli::try_parse_from(["malvin", "-q", "-g"]).expect("parse");
    assert!(cli.router.quiet);
    assert!(cli.router.gates);
}

#[test]
fn quiet_conflicts_with_do() {
    let err = Cli::try_parse_from(["malvin", "-q", "--do", "topic"]).expect_err("parse");
    let msg = err.to_string();
    assert!(
        msg.contains("cannot be used with") || msg.contains("--quiet") || msg.contains("-q"),
        "expected --quiet conflict with --do; got {msg}"
    );
}

#[test]
fn gates_only_short_q_is_router_quiet() {
    let cli = Cli::try_parse_from(["malvin", "-g", "-q"]).expect("parse");
    assert!(cli.router.quiet);
    assert!(cli.router.gates);
    assert!(cli.request.is_none());
}

#[test]
fn removed_inverse_flags_are_rejected() {
    for flag in ["--no-color", "--no-tee", "--no-markdown"] {
        let err = Cli::try_parse_from(["malvin", flag, "--do", "x"]).expect_err("parse");
        let msg = err.to_string();
        assert!(
            msg.contains("unexpected argument") || msg.contains(flag),
            "flag={flag} msg={msg}"
        );
    }
}
