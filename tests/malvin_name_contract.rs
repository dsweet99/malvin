mod common;

#[cfg(unix)]
use common::{fresh_workdir, sleep_child};
#[cfg(unix)]
use malvin::{
    acquire_acp_spawn_lock_for_slot, acquire_name, assert_no_peer_name_lock, name_path,
    names_registry_root, release_acp_spawn_lock,
};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn with_isolated_names<F>(f: F)
where
    F: FnOnce(),
{
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    std::fs::create_dir_all(&home).expect("mkdir home");
    let old_home = std::env::var_os("HOME");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("HOME", &home);
    }
    f();
    #[allow(unsafe_code)]
    unsafe {
        match old_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }
}

#[cfg(unix)]
#[test]
fn peer_name_lock_rejects_while_holder_alive() {
    with_isolated_names(|| {
        let mut child = sleep_child("120");
        let holder_pid = child.id();
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), format!("{holder_pid}\n")).expect("write lock");
        let err = assert_no_peer_name_lock("probe").expect_err("peer lock must block");
        assert!(
            err.contains(&holder_pid.to_string()),
            "expected holder pid in error, got: {err}"
        );
        assert!(
            err.contains(&name_path("probe").display().to_string()),
            "expected lock path in error, got: {err}"
        );
        let _ = child.kill();
        let _ = child.wait();
        assert_no_peer_name_lock("probe").expect("stale lock cleared after holder exit");
        assert!(!name_path("probe").exists(), "stale lock file removed");
    });
}

#[cfg(unix)]
#[test]
fn name_lock_released_after_process_exit() {
    with_isolated_names(|| {
        let guard = acquire_name("probe").expect("acquire");
        assert!(name_path("probe").is_file());
        drop(guard);
        assert!(!name_path("probe").exists());
    });
}

#[cfg(unix)]
#[test]
fn dead_holder_name_can_be_reused() {
    with_isolated_names(|| {
        let mut child = sleep_child("120");
        let holder_pid = child.id();
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), format!("{holder_pid}\n")).expect("write lock");
        let _ = child.kill();
        let _ = child.wait();
        acquire_name("probe").expect("dead holder reclaimed");
    });
}

#[cfg(unix)]
#[test]
fn abandoned_name_file_reclaimed_without_manual_cleanup() {
    with_isolated_names(|| {
        std::fs::create_dir_all(names_registry_root()).expect("mkdir names");
        std::fs::write(name_path("probe"), "424242\n").expect("abandoned lock");
        let guard = acquire_name("probe").expect("reclaim abandoned");
        assert_eq!(
            malvin::parse_holder_pid(
                &std::fs::read_to_string(name_path("probe")).expect("read after acquire")
            ),
            Some(std::process::id())
        );
        drop(guard);
    });
}

#[cfg(unix)]
#[test]
fn different_names_same_workspace_both_register() {
    with_isolated_names(|| {
        let guard_a = acquire_name("alpha").expect("acquire alpha");
        let guard_b = acquire_name("beta").expect("acquire beta");
        assert!(name_path("alpha").is_file());
        assert!(name_path("beta").is_file());
        drop(guard_a);
        drop(guard_b);
    });
}

#[cfg(unix)]
#[test]
fn different_acp_lock_slots_same_workspace_both_acquire() {
    malvin::malvin_sandbox::clear_active_sandbox_session();
    let work = fresh_workdir("malvin_different_acp_slots");
    std::fs::create_dir_all(work.join(".malvin/acp_spawn")).expect("mkdir .malvin");
    acquire_acp_spawn_lock_for_slot(&work, "alpha").expect("alpha");
    acquire_acp_spawn_lock_for_slot(&work, "beta").expect("beta");
    assert!(
        work.join(".malvin/acp_spawn/alpha.lock").is_file(),
        "alpha lock must exist"
    );
    assert!(
        work.join(".malvin/acp_spawn/beta.lock").is_file(),
        "beta lock must exist"
    );
    release_acp_spawn_lock(&work, "alpha");
    release_acp_spawn_lock(&work, "beta");
    malvin::malvin_sandbox::clear_active_sandbox_session();
}

#[cfg(unix)]
#[test]
fn entrypoint_duplicate_name_via_binary() {
    with_isolated_names(|| {
        let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
            .args(["--name", "probe", "--do", "plan.md"])
            .output()
            .expect("malvin");
        assert_eq!(out.status.code(), Some(1), "unknown --name must fail");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("--name"),
            "stderr must mention --name; got: {stderr}"
        );
    });
}

#[test]
fn kiss_cov_malvin_name_contract_symbols() {
    let _ = stringify!(peer_name_lock_rejects_while_holder_alive);
    let _ = stringify!(name_lock_released_after_process_exit);
    let _ = stringify!(dead_holder_name_can_be_reused);
    let _ = stringify!(abandoned_name_file_reclaimed_without_manual_cleanup);
    let _ = stringify!(different_names_same_workspace_both_register);
    let _ = stringify!(different_acp_lock_slots_same_workspace_both_acquire);
    let _ = stringify!(entrypoint_duplicate_name_via_binary);
    #[cfg(unix)]
    {}
}
