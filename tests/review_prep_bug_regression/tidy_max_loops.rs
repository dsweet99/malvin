use super::helpers::manifest_root;

#[test]
fn kiss_stringify_cov_tests_effective_tidy_max_loops_not_raw_zero_field() {
    let src = std::fs::read_to_string(manifest_root().join("src/cli/kiss_stringify_cov.rs"))
        .expect("read kiss_stringify_cov.rs");
    assert!(
        src.contains("effective_tidy_max_loops(0), 1)"),
        "bug: kiss_stringify_cov must assert effective_tidy_max_loops(0) == 1, not only \
         TidyArgs {{ max_loops: 0 }}"
    );
    assert!(
        !src.contains("assert_eq!(args.max_loops, 0)"),
        "bug: kiss_stringify_cov still asserts raw max_loops == 0 while claiming runtime \
         normalization"
    );
}

#[test]
fn tidy_interleaved_loop_uses_effective_tidy_max_loops_helper() {
    let src =
        std::fs::read_to_string(manifest_root().join("src/cli/tidy_flow/interleaved_loop.inc"))
            .expect("read interleaved_loop.inc");
    assert!(
        src.contains("effective_tidy_max_loops(max_loops)"),
        "bug: tidy interleaved loop must normalize zero max_loops via \
         effective_tidy_max_loops, not ad-hoc max_loops.max(1) only in tests"
    );
}
