use super::*;
use std::path::Path;

#[path = "session_name_kiss_cov_tests.rs"]
mod session_name_kiss_cov_tests;

#[path = "auto_name.rs"]
mod auto_name;

pub(super) fn with_isolated_names<F>(f: F)
where
    F: FnOnce(&Path),
{
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let root = names_registry_root();
        if root.exists() {
            let _ = std::fs::remove_dir_all(&root);
        }
        f(&root);
    });
}

pub(super) fn sleep_child(seconds: &str) -> std::process::Child {
    let mut cmd = crate::malvin_sandbox::malvin_std_command("sleep");
    cmd.arg(seconds);
    cmd.spawn().expect("spawn sleep")
}

fn name_path_under_malvin_home() {
    with_isolated_names(|root| {
        assert_eq!(name_path("probe"), root.join("probe"));
    });
}

fn validate_name_rejects_empty() {
    assert!(validate_name("").is_err());
}

fn validate_name_rejects_path_chars() {
    assert!(validate_name("/").is_err());
    assert!(validate_name("..").is_err());
    assert!(validate_name("has space").is_err());
}

fn validate_name_accepts_alnum_dash_dot() {
    assert!(validate_name("my-run_1").is_ok());
}

fn acquire_creates_pid_file() {
    with_isolated_names(|_| {
        let guard = acquire_name("probe").expect("acquire");
        let path = name_path("probe");
        assert!(path.is_file());
        assert_eq!(
            parse_holder_pid(&std::fs::read_to_string(&path).expect("read")),
            Some(std::process::id())
        );
        drop(guard);
    });
}

#[cfg(unix)]
fn acquire_rejects_live_peer() {
    with_isolated_names(|_| {
        let mut child = sleep_child("120");
        let holder_pid = child.id();
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), format!("{holder_pid}\n")).expect("write peer");
        let err = acquire_name("probe").expect_err("live peer blocks");
        assert!(err.contains(&holder_pid.to_string()), "err={err}");
        let _ = child.kill();
        let _ = child.wait();
    });
}

fn acquire_reclaims_stale_dead_pid() {
    with_isolated_names(|_| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), "424242\n").expect("write dead pid");
        let guard = acquire_name("probe").expect("reclaim stale");
        assert_eq!(
            parse_holder_pid(
                &std::fs::read_to_string(name_path("probe")).expect("read after acquire")
            ),
            Some(std::process::id())
        );
        drop(guard);
    });
}

fn acquire_clears_invalid_contents() {
    with_isolated_names(|_| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), "not-a-pid").expect("write invalid");
        acquire_name("probe").expect("invalid cleared");
    });
}

fn acquire_reclaims_empty_file() {
    with_isolated_names(|_| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), "").expect("write empty");
        acquire_name("probe").expect("empty reclaimed");
    });
}

#[cfg(unix)]
fn acquire_reclaims_whitespace_pid() {
    with_isolated_names(|_| {
        let mut child = sleep_child("120");
        let holder_pid = child.id();
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), format!("  {holder_pid} \n")).expect("write");
        assert!(acquire_name("probe").is_err(), "trimmed live pid blocks");
        let _ = child.kill();
        let _ = child.wait();
        acquire_name("probe").expect("dead trimmed pid reclaimed");
    });
}

fn clear_stale_removes_non_regular_path() {
    with_isolated_names(|_| {
        let path = name_path("probe");
        std::fs::create_dir_all(&path).expect("directory at name path");
        acquire_name("probe").expect("non-regular removed");
    });
}

fn acquire_cleans_up_on_write_failure() {
    with_isolated_names(|_| {
        let err = acquire_name_with_write("probe", |_| {
            Err(std::io::Error::other("mock write failure"))
        })
        .expect_err("write fails");
        assert!(err.contains("mock write failure"), "err={err}");
        assert!(!name_path("probe").exists(), "partial file removed");
    });
}

fn release_removes_own_file() {
    with_isolated_names(|_| {
        let guard = acquire_name("probe").expect("acquire");
        let path = name_path("probe");
        assert!(path.is_file());
        drop(guard);
        assert!(!path.exists());
    });
}

fn release_preserves_foreign_file() {
    with_isolated_names(|_| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), "424242\n").expect("foreign pid");
        release_name("probe");
        assert!(name_path("probe").exists(), "foreign file preserved");
    });
}

fn parse_holder_pid_rejects_garbage() {
    assert_eq!(parse_holder_pid(""), None);
    assert_eq!(parse_holder_pid("abc"), None);
    assert_eq!(parse_holder_pid("12 34"), None);
}

#[test]
fn kiss_bundled_session_name_tests() {
    name_path_under_malvin_home();
    validate_name_rejects_empty();
    validate_name_rejects_path_chars();
    validate_name_accepts_alnum_dash_dot();
    acquire_creates_pid_file();
    acquire_rejects_live_peer();
    acquire_reclaims_stale_dead_pid();
    acquire_clears_invalid_contents();
    acquire_reclaims_empty_file();
    acquire_reclaims_whitespace_pid();
    clear_stale_removes_non_regular_path();
    acquire_cleans_up_on_write_failure();
    release_removes_own_file();
    release_preserves_foreign_file();
    parse_holder_pid_rejects_garbage();
}
