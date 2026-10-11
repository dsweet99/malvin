use crate::cli::Cli;
use crate::cli::config_defaults::parse_cli_with_config_defaults;
use clap::Parser;
use malvin::test_support::test_utils::with_isolated_home;
#[test]
fn global_quiet_long_and_short_parse_and_quiet_parses_on_router_wrappers() {
    {
        let long = Cli::try_parse_from(["malvin", "--quiet", "hello"]).expect("parse");
        assert!(long.router.dm_stdout_only());
        assert_eq!(long.first_request().map(String::as_str), Some("hello"));
        let short = Cli::try_parse_from(["malvin", "-q", "hello"]).expect("parse");
        assert!(short.router.dm_stdout_only());
    }
    {
        let cli = Cli::try_parse_from(["malvin", "-q", "-g"]).expect("parse");
        assert!(cli.router.dm_stdout_only());
        assert!(cli.router.gates);
    }
}

#[test]
fn removed_do_flag_is_rejected_beside_quiet() {
    let mut err = None;
    with_isolated_home(|_work| {
        err = Some(
            parse_cli_with_config_defaults(["malvin", "-q", "--do", "topic"])
                .expect_err("parse")
                .to_string(),
        );
    });
    let msg = err.expect("err");
    assert!(
        msg.contains("unexpected") || msg.contains("--do"),
        "expected removed --do rejected; got {msg}"
    );
}

#[test]
fn gates_only_short_q_is_router_quiet() {
    let cli = Cli::try_parse_from(["malvin", "-g", "-q"]).expect("parse");
    assert!(cli.router.dm_stdout_only());
    assert!(cli.router.gates);
    assert!(!cli.has_request());
}

#[test]
fn very_quiet_long_and_double_short_print_nothing() {
    let long = Cli::try_parse_from(["malvin", "--very-quiet", "hello"]).expect("parse");
    assert!(long.router.prints_nothing());
    assert!(!long.router.dm_stdout_only());
    assert_eq!(long.first_request().map(String::as_str), Some("hello"));

    let short = Cli::try_parse_from(["malvin", "-qq", "hello"]).expect("parse");
    assert!(short.router.prints_nothing());
    assert!(!short.router.dm_stdout_only());

    let clustered = Cli::try_parse_from(["malvin", "-gqq"]).expect("parse");
    assert!(clustered.router.prints_nothing());
    assert!(clustered.router.gates);

    let quiet_then_very =
        Cli::try_parse_from(["malvin", "-q", "--very-quiet", "hello"]).expect("parse");
    assert!(quiet_then_very.router.prints_nothing());
    assert!(!quiet_then_very.router.dm_stdout_only());
}

#[test]
fn removed_inverse_flags_are_rejected() {
    for flag in ["--no-color", "--no-tee", "--no-markdown"] {
        let err = Cli::try_parse_from(["malvin", flag, "x"]).expect_err("parse");
        let msg = err.to_string();
        assert!(
            msg.contains("unexpected argument") || msg.contains(flag),
            "flag={flag} msg={msg}"
        );
    }
}
