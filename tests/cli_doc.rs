//! Smoke: `malvin --doc` prints embedded top-level documentation.

const MALVIN_MD: &str = include_str!("../default_prompts/docs/malvin.md");

#[test]
fn malvin_doc_prints_full_malvin_md() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .arg("--doc")
        .output()
        .expect("spawn malvin --doc");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout.as_slice(), MALVIN_MD.as_bytes());
}

#[test]
fn bare_malvin_exits_nonzero() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .output()
        .expect("spawn malvin");
    assert!(!output.status.success());
}
