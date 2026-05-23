#[test]
fn time_format_timestamp_symbol_lives_in_one_module() {
    let time_format = include_str!("time_format.rs");
    let output_mod = include_str!("output/mod.rs");
    assert!(
        time_format.contains("pub fn timestamp_now_string()"),
        "time_format.rs is the canonical home for timestamp_now_string"
    );
    assert!(
        !output_mod.contains("pub(crate) fn timestamp_now_string()"),
        "output must not define a duplicate timestamp_now_string wrapper"
    );
    assert!(
        output_mod.contains("pub(crate) use crate::time_format::timestamp_now_string"),
        "output must re-export timestamp_now_string from time_format"
    );
}

#[test]
fn handshake_types_inc_must_pass_focused_kiss_per_file_coverage() {
    let status = std::process::Command::new("kiss")
        .args(["check", ".", "src/acp/handshake_types.inc"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("spawn kiss");
    assert!(
        status.success(),
        "handshake_types.inc must meet kiss per-file 90% threshold; focused check still fails after gate-fix"
    );
}
