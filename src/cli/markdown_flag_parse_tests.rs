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
        ["malvin", "-q", "-g"].as_slice(),
        ["malvin", "-q", "write", "topic"].as_slice(),
    ] {
        let cli = Cli::try_parse_from(argv).expect("parse");
        assert!(cli.shared.quiet, "argv={argv:?}");
    }
}

#[test]
fn gates_only_short_q_is_global_quiet() {
    let cli = Cli::try_parse_from(["malvin", "-g", "-q"]).expect("parse");
    assert!(cli.shared.quiet);
    assert!(cli.shared.gates);
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
