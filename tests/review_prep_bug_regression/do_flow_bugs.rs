#[test]
fn do_flow_verbose_default_test_name_must_match_assertion() {
    let src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/do_flow_b.inc"
    ));
    let claims_default_verbose =
        src.contains("fn cli_defaults_to_verbose_and_accepts_verbose_flags");
    let asserts_not_verbose_on_default = src.contains("try_parse_from([\"malvin\", \"do\", \"default\"])")
        && src.contains("assert!(!cli.shared.verbose)");
    assert!(
        !(claims_default_verbose && asserts_not_verbose_on_default),
        "bug: test name says defaults to verbose but asserts !cli.shared.verbose on default parse; \
         rename the test or fix the assertion"
    );
}
