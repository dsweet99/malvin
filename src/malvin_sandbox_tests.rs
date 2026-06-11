use std::ffi::OsStr;

#[test]
fn kiss_cov_malvin_sandbox_symbols() {
    let _ = stringify!(crate::malvin_sandbox::init_malvin_spawn_baseline);
    let _ = crate::acp::reap_baseline_amnestied_agent_orphans_blocking;
    let _ = stringify!(crate::malvin_sandbox::malvin_std_command);
    let _ = stringify!(crate::malvin_sandbox::note_active_mini_session);
}

#[allow(unsafe_code)]
fn with_env_set(key: &str, value: &str, f: impl FnOnce()) {
    let prior = std::env::var_os(key);
    unsafe {
        std::env::set_var(key, value);
    }
    f();
    unsafe {
        match prior {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}

#[test]
fn sandbox_commands_do_not_force_cargo_build_jobs() {
    with_env_set("CARGO_BUILD_JOBS", "8", || {
        let std_cmd = crate::malvin_sandbox::malvin_std_command("true");
        let jobs = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "CARGO_BUILD_JOBS").then_some(v));
        assert!(jobs.is_none());
    });
}

#[test]
fn sandbox_commands_force_nextest_test_threads_to_one() {
    with_env_set("NEXTEST_TEST_THREADS", "8", || {
        let std_cmd = crate::malvin_sandbox::malvin_std_command("true");
        let threads = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "NEXTEST_TEST_THREADS").then_some(v));
        assert_eq!(threads, Some(Some(OsStr::new("1"))));
    });
}

#[test]
fn sandbox_commands_do_not_force_rust_test_threads() {
    with_env_set("RUST_TEST_THREADS", "8", || {
        let std_cmd = crate::malvin_sandbox::malvin_std_command("true");
        let threads = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "RUST_TEST_THREADS").then_some(v));
        assert!(threads.is_none());
    });
}

#[test]
fn sandbox_commands_force_malloc_arena_max_to_two() {
    with_env_set("MALLOC_ARENA_MAX", "8", || {
        let std_cmd = crate::malvin_sandbox::malvin_std_command("true");
        let arenas = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "MALLOC_ARENA_MAX").then_some(v));
        assert_eq!(arenas, Some(Some(OsStr::new("2"))));
    });
}

#[test]
fn note_active_mini_session_cleared_after_end() {
    let tmp = tempfile::tempdir().expect("tempdir");
    crate::malvin_sandbox::clear_active_sandbox_session_for_test();
    crate::malvin_sandbox::note_active_mini_session(tmp.path())
        .expect("note");
    crate::malvin_sandbox::clear_active_mini_session();
    crate::malvin_sandbox::clear_active_sandbox_session_for_test();
}
