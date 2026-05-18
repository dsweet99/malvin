use super::helpers::manifest_root;

#[test]
fn helpers_tests_cover_effective_tidy_max_loops_not_raw_zero_field() {
    let src = std::fs::read_to_string(manifest_root().join("src/cli/tidy_flow/helpers_tests.inc"))
        .expect("read helpers_tests.inc");
    assert!(
        src.contains("effective_tidy_max_loops(0), 1)")
            || src.contains("effective_tidy_max_loops(0),1)"),
        "bug: helpers_tests must assert effective_tidy_max_loops(0) == 1"
    );
}

#[test]
fn tidy_interleaved_loop_uses_effective_tidy_max_loops_helper() {
    let src =
        std::fs::read_to_string(manifest_root().join("src/cli/tidy_flow/interleaved_loop.inc"))
            .expect("read interleaved_loop.inc");
    assert!(
        src.contains("effective_tidy_max_loops(max_loops)"),
        "bug: tidy interleaved loop must normalize zero max_loops via effective_tidy_max_loops"
    );
}
