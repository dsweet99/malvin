#![cfg_attr(test, allow(unsafe_code))]

use super::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, bytes).unwrap();
}

fn cursor_ready_tree(root: &Path) -> PathBuf {
    let src = root.join(BRIDGES[0].dir_name);
    let marker = BRIDGES[0].package_marker;
    write_file(&src.join("package.json"), b"{}");
    write_file(&src.join("node_modules").join(marker), b"{}");
    write_file(&src.join("dist").join("bridge.js"), b"1");
    src
}

#[test]
fn bridges_declare_cursor_and_prime_markers() {
    assert_eq!(BRIDGES.len(), 2);
    assert_eq!(BRIDGES[0].package_marker, "@cursor/sdk/package.json");
    assert_eq!(BRIDGES[1].package_marker, "prime-agent/package.json");
    assert!(BRIDGES[0].min_node >= (22, 13));
    assert!(BRIDGES[1].min_node >= (22, 8));
}

#[test]
fn fnv1a64_is_stable() {
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_ne!(fnv1a64(b"a"), fnv1a64(b"b"));
    assert_eq!(fnv1a64(b"lock"), fnv1a64(b"lock"));
}

#[test]
fn parse_node_version_accepts_v_prefix() {
    assert_eq!(npm::parse_node_version("v22.13.0"), Some((22, 13)));
    assert_eq!(npm::parse_node_version("22.8.1"), Some((22, 8)));
    assert_eq!(npm::parse_node_version("nope"), None);
}

#[test]
fn which_finds_sh_on_unix() {
    let _g = crate::test_utils::test_env_lock();
    assert!(npm::which("sh").is_some());
    assert!(npm::which("definitely-not-a-real-bin-xyz").is_none());
}

#[test]
fn sdk_share_dir_uses_home() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempdir().unwrap();
    crate::acp::with_env("HOME", Some(tmp.path().to_str().unwrap()), || {
        assert_eq!(
            sdk_share_dir(),
            tmp.path().join(".malvin_home").join("sdk-bridges")
        );
    });
}

#[test]
fn copy_dir_recursive_copies_nested_files() {
    let tmp = tempdir().unwrap();
    let from = tmp.path().join("from");
    let to = tmp.path().join("to");
    write_file(&from.join("a.txt"), b"a");
    write_file(&from.join("sub").join("b.txt"), b"b");
    copy::copy_dir_recursive(&from, &to).unwrap();
    assert_eq!(fs::read(to.join("a.txt")).unwrap(), b"a");
    assert_eq!(fs::read(to.join("sub").join("b.txt")).unwrap(), b"b");
}

fn sync_fixture_with_dist(tmp: &TempDir) -> (PathBuf, PathBuf) {
    let src = tmp.path().join("src_bridge");
    let dest = tmp.path().join("dest_bridge");
    write_file(&src.join("package.json"), b"{}");
    write_file(&src.join("package-lock.json"), b"{}");
    write_file(&src.join("dist").join("bridge.js"), b"ok");
    (src, dest)
}

#[test]
fn sync_bridge_payload_copies_dist_when_present() {
    let tmp = tempdir().unwrap();
    let (src, dest) = sync_fixture_with_dist(&tmp);
    sync::sync_bridge_payload(&src, &dest, "src_bridge");
    assert_eq!(fs::read(dest.join("dist").join("bridge.js")).unwrap(), b"ok");
}

fn sync_fixture_sources(tmp: &TempDir) -> (PathBuf, PathBuf) {
    let src = tmp.path().join("src_bridge");
    let dest = tmp.path().join("dest_bridge");
    write_file(&src.join("package.json"), b"{}");
    write_file(&src.join("package-lock.json"), b"{}");
    write_file(&src.join("tsconfig.json"), b"{}");
    write_file(&src.join("src").join("bridge.ts"), b"x");
    (src, dest)
}

#[test]
fn sync_bridge_payload_copies_sources_without_dist() {
    let tmp = tempdir().unwrap();
    let (src, dest) = sync_fixture_sources(&tmp);
    sync::sync_bridge_payload(&src, &dest, "src_bridge");
    assert!(dest.join("src").join("bridge.ts").is_file());
    assert!(dest.join("tsconfig.json").is_file());
}

#[test]
fn in_tree_ready_requires_marker_and_dist() {
    let tmp = tempdir().unwrap();
    let src = tmp.path().join("cursor-sdk-bridge");
    let bridge = &BRIDGES[0];
    assert!(!in_tree_bridge_ready(&src, bridge));
    write_file(&src.join("node_modules").join(bridge.package_marker), b"{}");
    assert!(!in_tree_bridge_ready(&src, bridge));
    write_file(&src.join("dist").join("bridge.js"), b"1");
    assert!(in_tree_bridge_ready(&src, bridge));
}

fn prime_share_fixture(tmp: &TempDir) -> (PathBuf, PathBuf, &'static [u8]) {
    let src = tmp.path().join("src");
    let dest = tmp.path().join("dest");
    let lock = b"{\"lock\":1}";
    write_file(&src.join("package-lock.json"), lock);
    let bridge = &BRIDGES[1];
    write_file(&dest.join("node_modules").join(bridge.package_marker), b"{}");
    write_file(&dest.join("dist").join("bridge.js"), b"1");
    (src, dest, lock)
}

#[test]
fn share_bridge_ready_checks_stamp() {
    let tmp = tempdir().unwrap();
    let (src, dest, lock) = prime_share_fixture(&tmp);
    let bridge = &BRIDGES[1];
    assert!(!share_bridge_ready(&src, &dest, bridge));
    let stamp = format!("{:x}", fnv1a64(lock));
    write_file(&dest.join(".malvin-npm-stamp"), format!("{stamp}\n").as_bytes());
    assert!(share_bridge_ready(&src, &dest, bridge));
}

#[test]
fn ensure_bridge_reuses_in_tree_ready_tree() {
    let tmp = tempdir().unwrap();
    let _ = cursor_ready_tree(tmp.path());
    ensure_bridge(tmp.path(), &BRIDGES[0]);
}

#[test]
fn run_build_script_skips_when_env_set() {
    let _g = crate::test_utils::test_env_lock();
    crate::acp::with_env("MALVIN_SKIP_SDK_BRIDGES", Some("1"), run_build_script);
}

#[test]
fn run_build_script_skips_on_docs_rs() {
    let _g = crate::test_utils::test_env_lock();
    crate::acp::with_env("DOCS_RS", Some("1"), || {
        crate::acp::with_env("MALVIN_SKIP_SDK_BRIDGES", None, run_build_script);
    });
}

#[test]
fn write_stamp_and_verify_install_round_trip() {
    let tmp = tempdir().unwrap();
    let dest = tmp.path().join("dest");
    let bridge = &BRIDGES[0];
    write_file(&dest.join("package-lock.json"), b"abc");
    write_file(&dest.join("node_modules").join(bridge.package_marker), b"{}");
    write_file(&dest.join("dist").join("bridge.js"), b"1");
    verify_install(&dest, bridge);
    write_stamp(&dest);
    assert!(dest.join(".malvin-npm-stamp").is_file());
}

#[test]
fn resolve_npm_prefers_npm_env() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempdir().unwrap();
    let fake = tmp.path().join("npm");
    write_file(&fake, b"#!/bin/sh\n");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake, perms).unwrap();
    }
    crate::acp::with_env("npm", Some(fake.to_str().unwrap()), || {
        assert_eq!(npm::resolve_npm(), fake);
    });
}

#[test]
fn check_node_version_accepts_current_node() {
    let _g = crate::test_utils::test_env_lock();
    if npm::which("node").is_none() {
        return;
    }
    check_node_version(&BRIDGES[1]);
}
