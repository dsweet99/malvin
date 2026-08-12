use std::os::unix::fs::PermissionsExt;

use super::discover::{
    parse_pi_version, pi_missing_binary_message, pi_version_ok, resolve_pi_bin, PI_MIN_VERSION,
};
use super::models_list::parse_list_models_table;

#[test]
fn resolve_pi_bin_honors_malvin_pi_override() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let fake = write_exec_script(dir.path(), "fake-pi", "#!/bin/sh\necho fake\n");
    crate::acp::with_env("MALVIN_PI", Some(fake.to_str().expect("utf8")), || {
        let got = resolve_pi_bin().expect("resolve");
        assert_eq!(got, fake);
    });
}

#[test]
fn resolve_pi_bin_rejects_missing_malvin_pi() {
    let missing = tempfile::tempdir()
        .expect("tmpdir")
        .path()
        .join("no-such-pi");
    crate::acp::with_env("MALVIN_PI", Some(missing.to_str().expect("utf8")), || {
        let err = resolve_pi_bin().expect_err("missing");
        assert!(err.contains("MALVIN_PI"));
        assert!(
            err.contains("does not bundle")
                || err.contains(pi_missing_binary_message().as_str())
                || err.contains("Install")
        );
    });
}

#[test]
fn resolve_pi_bin_rejects_non_executable_malvin_pi() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let fake = dir.path().join("pi-noexec");
    std::fs::write(&fake, "#!/bin/sh\n").expect("write");
    let mut perms = std::fs::metadata(&fake).expect("meta").permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&fake, perms).expect("chmod");
    crate::acp::with_env("MALVIN_PI", Some(fake.to_str().expect("utf8")), || {
        let err = resolve_pi_bin().expect_err("non-exec");
        assert!(err.contains("not executable"));
    });
}

#[test]
fn parse_list_models_skips_header() {
    let text = "\
provider    model                                          context
openai      gpt-4o                                         128K
openrouter  anthropic/claude-3-haiku                       200K
Showing 2 of 95 providers. Run `pi --list-providers` to see all.
";
    let rows = parse_list_models_table(text);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "openai/gpt-4o");
    assert_eq!(rows[1].id, "openrouter/anthropic/claude-3-haiku");
    assert!(rows.iter().all(|r| !r.id.to_ascii_lowercase().contains("showing")));
}

#[test]
fn pi_version_ok_and_missing_hint() {
    assert!(pi_missing_binary_message().contains("MALVIN_PI"));
    assert_eq!(PI_MIN_VERSION, (0, 1, 23));
    assert_eq!(
        parse_pi_version("pi 0.1.23 (590d6189 2026-08-11T00:00:10Z)"),
        Some((0, 1, 23))
    );
    let dir = tempfile::tempdir().expect("tmpdir");
    let ok_bin = write_exec_script(dir.path(), "pi-ok", "#!/bin/sh\necho 'pi 0.1.23'\n");
    assert!(pi_version_ok(&ok_bin).is_ok());
    let old_bin = write_exec_script(dir.path(), "pi-old", "#!/bin/sh\necho 'pi 0.1.22'\n");
    assert!(pi_version_ok(&old_bin).expect_err("old").contains("too old"));
    let bad_bin = write_exec_script(dir.path(), "pi-bad", "#!/bin/sh\nexit 1\n");
    assert!(pi_version_ok(&bad_bin).is_err());
}

fn write_exec_script(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write");
    let mut perms = std::fs::metadata(&path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod");
    path
}
