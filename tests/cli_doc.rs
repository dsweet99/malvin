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
fn malvin_code_without_request_shows_short_usage_and_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_malvin");
    let bare = std::process::Command::new(bin)
        .args(["code"])
        .output()
        .expect("spawn malvin code");
    let help = std::process::Command::new(bin)
        .args(["code", "--help"])
        .output()
        .expect("spawn malvin code --help");
    assert!(
        bare.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&bare.stderr)
    );
    assert!(help.status.success());
    let bare_s = String::from_utf8_lossy(&bare.stdout);
    let help_s = String::from_utf8_lossy(&help.stdout);
    assert_ne!(
        bare.stdout, help.stdout,
        "malvin code must not duplicate full --help"
    );
    assert!(bare_s.contains("Write code"), "code stdout: {bare_s}");
    assert!(
        bare_s.contains("Usage: malvin code [REQUEST]"),
        "code stdout must show REQUEST usage: {bare_s}"
    );
    assert!(
        bare_s.contains("malvin code --help"),
        "code stdout must point to --help: {bare_s}"
    );
    assert!(
        !bare_s.contains("Options:"),
        "code stdout must omit options: {bare_s}"
    );
    assert!(
        help_s.contains("Options:"),
        "full help must list options: {help_s}"
    );
    assert!(
        help_s.contains("--max-loops"),
        "full help must list code flags: {help_s}"
    );
}

#[test]
fn malvin_inspire_without_request_shows_short_usage_and_exits_zero() {
    let bin = env!("CARGO_BIN_EXE_malvin");
    let bare = std::process::Command::new(bin)
        .args(["inspire"])
        .output()
        .expect("spawn malvin inspire");
    let help = std::process::Command::new(bin)
        .args(["inspire", "--help"])
        .output()
        .expect("spawn malvin inspire --help");
    assert!(
        bare.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&bare.stderr)
    );
    assert!(help.status.success());
    let bare_s = String::from_utf8_lossy(&bare.stdout);
    let help_s = String::from_utf8_lossy(&help.stdout);
    assert_ne!(
        bare.stdout, help.stdout,
        "malvin inspire must not duplicate full --help"
    );
    assert!(bare_s.contains("Be creative"), "inspire stdout: {bare_s}");
    assert!(
        bare_s.contains("Usage: malvin inspire [REQUEST]"),
        "inspire stdout must show REQUEST usage: {bare_s}"
    );
    assert!(
        bare_s.contains("malvin inspire --help"),
        "inspire stdout must point to --help: {bare_s}"
    );
    assert!(
        !bare_s.contains("Options:"),
        "inspire stdout must omit options: {bare_s}"
    );
    assert!(
        help_s.contains("Options:"),
        "full help must list options: {help_s}"
    );
    assert!(
        help_s.contains("--model"),
        "full help must list inspire flags: {help_s}"
    );
}

#[test]
fn bare_malvin_shows_commands_only_and_exits_zero() {
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
    let bare_s = String::from_utf8_lossy(&bare.stdout);
    let help_s = String::from_utf8_lossy(&help.stdout);
    assert_ne!(
        bare.stdout, help.stdout,
        "bare malvin must not duplicate full --help"
    );
    assert!(bare_s.contains("Commands:"), "bare stdout: {bare_s}");
    assert!(
        bare_s.contains("[COMMAND|REQUEST]"),
        "bare stdout must show [COMMAND|REQUEST] usage: {bare_s}"
    );
    assert!(
        bare_s.contains("malvin --help"),
        "bare stdout must point to --help: {bare_s}"
    );
    assert!(
        !bare_s.contains("Options:"),
        "bare stdout must omit options: {bare_s}"
    );
    assert!(
        help_s.contains("Options:"),
        "full help must list options: {help_s}"
    );
    assert!(
        help_s.contains("--no-color"),
        "full help must list global flags: {help_s}"
    );
}
