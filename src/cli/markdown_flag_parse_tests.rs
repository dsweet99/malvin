use crate::cli::Cli;
use clap::Parser;

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
