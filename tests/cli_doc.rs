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
fn bare_malvin_matches_help_and_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_malvin");
    let bare = std::process::Command::new(bin)
        .output()
        .expect("spawn malvin");
    let help = std::process::Command::new(bin)
        .arg("--help")
        .output()
        .expect("spawn malvin --help");
    assert!(
        bare.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&bare.stderr)
    );
    assert!(help.status.success());
    assert_eq!(bare.stdout, help.stdout);
    assert_eq!(bare.stderr, help.stderr);
}
