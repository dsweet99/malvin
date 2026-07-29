//! Smoke: `malvin --doc` prints embedded top-level documentation.

const MALVIN_MD: &str = include_str!("../default_prompts/docs/malvin.md");
const ROUTER_MD: &str = include_str!("../default_prompts/docs/router.md");

fn isolated_home() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("home")).expect("mkdir home");
    tmp
}

fn malvin_cmd(home_root: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.env("HOME", home_root.join("home"));
    cmd
}

#[test]
fn malvin_doc_prints_overview_then_router() {
    let tmp = isolated_home();
    let output = malvin_cmd(tmp.path())
        .arg("--doc")
        .output()
        .expect("spawn malvin --doc");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = format!("{MALVIN_MD}\n---\n\n{ROUTER_MD}");
    assert_eq!(output.stdout.as_slice(), expected.as_bytes());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.starts_with(MALVIN_MD));
    assert!(text.contains(ROUTER_MD));
    assert!(text.contains("# malvin (default route)"));
}

#[test]
fn malvin_code_is_deprecated() {
    let tmp = isolated_home();
    let bin_home = tmp.path();
    for args in [
        &["code"][..],
        &["code", "plan.md"][..],
        &["code", "--help"][..],
        &["code", "--doc"][..],
    ] {
        let out = malvin_cmd(bin_home)
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("spawn malvin {args:?}: {e}"));
        assert_eq!(
            out.status.code(),
            Some(1),
            "malvin {args:?} must exit 1; stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("deprecated"),
            "malvin {args:?} stderr must mention deprecation: {stderr}"
        );
    }
}

#[test]
fn malvin_inspire_without_request_shows_short_usage_and_exits_zero() {
    let tmp = isolated_home();
    let bare = malvin_cmd(tmp.path())
        .args(["inspire"])
        .output()
        .expect("spawn malvin inspire");
    let help = malvin_cmd(tmp.path())
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
    let tmp = isolated_home();
    let bare = malvin_cmd(tmp.path())
        .output()
        .expect("spawn malvin");
    let help = malvin_cmd(tmp.path())
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
        bare_s.contains("[REQUEST]"),
        "bare stdout must show [REQUEST] usage: {bare_s}"
    );
    assert!(
        bare_s.contains("<COMMAND>"),
        "bare stdout must show <COMMAND> usage: {bare_s}"
    );
    assert!(
        bare_s.contains("tidy"),
        "bare stdout must list tidy subcommand: {bare_s}"
    );
    assert!(
        !bare_s.lines().any(|line| line.starts_with("  kpop ")),
        "bare stdout must omit removed kpop subcommand: {bare_s}"
    );
    assert!(
        !bare_s.lines().any(|line| line.starts_with("  code ")),
        "bare stdout must omit deprecated code subcommand: {bare_s}"
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
