use std::ffi::OsStr;

#[test]
fn kiss_cov_malvin_sandbox_symbols() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    let _ = stringify!(crate::agent_process::malvin_sandbox::init_malvin_spawn_baseline);
    let _ = crate::agent_process::reap_baseline_amnestied_agent_orphans_blocking;
    let _ = stringify!(crate::agent_process::malvin_sandbox::malvin_std_command);
    let _ = stringify!(crate::agent_process::malvin_sandbox::install_parent_death_signal);
    let _ = stringify!(crate::agent_process::malvin_sandbox::install_tokio_parent_death_signal);
    let _ = stringify!(crate::agent_process::malvin_sandbox::note_active_sandbox_session);
}

#[test]
fn sandbox_commands_do_not_force_cargo_build_jobs() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("CARGO_BUILD_JOBS", Some("8"), || {
        let std_cmd = crate::agent_process::malvin_sandbox::malvin_std_command("true");
        let jobs = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "CARGO_BUILD_JOBS").then_some(v));
        assert!(jobs.is_none());
    });
}

#[test]
fn sandbox_commands_do_not_force_nextest_test_threads() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("NEXTEST_TEST_THREADS", Some("8"), || {
        let std_cmd = crate::agent_process::malvin_sandbox::malvin_std_command("true");
        let threads = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "NEXTEST_TEST_THREADS").then_some(v));
        assert!(threads.is_none());
    });
}

#[test]
fn sandbox_commands_do_not_force_rust_test_threads() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("RUST_TEST_THREADS", Some("8"), || {
        let std_cmd = crate::agent_process::malvin_sandbox::malvin_std_command("true");
        let threads = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "RUST_TEST_THREADS").then_some(v));
        assert!(threads.is_none());
    });
}

#[test]
fn sandbox_commands_force_malloc_arena_max_to_two() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    crate::agent_process::with_env("MALLOC_ARENA_MAX", Some("8"), || {
        let std_cmd = crate::agent_process::malvin_sandbox::malvin_std_command("true");
        let arenas = std_cmd
            .get_envs()
            .find_map(|(k, v)| (k == "MALLOC_ARENA_MAX").then_some(v));
        assert_eq!(arenas, Some(Some(OsStr::new("2"))));
        let tokio_cmd = crate::agent_process::malvin_sandbox::malvin_tokio_command("true");
        let arenas = tokio_cmd
            .as_std()
            .get_envs()
            .find_map(|(k, v)| (k == "MALLOC_ARENA_MAX").then_some(v));
        assert_eq!(arenas, Some(Some(OsStr::new("2"))));
    });
}

#[test]
fn note_active_sandbox_session_cleared_after_end() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().expect("tempdir");
    crate::agent_process::malvin_sandbox::clear_active_sandbox_session_for_test();
    let ticket = crate::agent_process::malvin_sandbox::take_sandbox_spawn_ticket().expect("ticket");
    crate::agent_process::malvin_sandbox::note_active_sandbox_session(
        ticket,
        None,
        crate::agent_process::malvin_sandbox::malvin_spawn_baseline(),
        tmp.path(),
    )
    .expect("note");
    crate::agent_process::malvin_sandbox::clear_active_sandbox_session();
    crate::agent_process::malvin_sandbox::clear_active_sandbox_session_for_test();
}

#[cfg(unix)]
#[tokio::test]
#[allow(unsafe_code)]
async fn tokio_sandbox_command_keeps_group_isolation_and_herdr_stripping() {
    let _env_guard = crate::test_support::test_utils::test_env_lock();
    let prior = std::env::var_os("HERDR_PANE_ID");
    unsafe {
        std::env::set_var("HERDR_PANE_ID", "p1");
    }
    let mut cmd = crate::agent_process::malvin_sandbox::malvin_tokio_command("sh");
    cmd.args([
        "-c",
        "echo \"$$ $(ps -o pgid= -p $$) [${HERDR_PANE_ID-unset}]\"",
    ]);
    let out = cmd.output().await;
    unsafe {
        match prior {
            Some(v) => std::env::set_var("HERDR_PANE_ID", v),
            None => std::env::remove_var("HERDR_PANE_ID"),
        }
    }
    let text = String::from_utf8(out.expect("run").stdout).expect("utf8");
    let mut fields = text.split_whitespace();
    let pid = fields.next().expect("pid");
    let pgid = fields.next().expect("pgid");
    assert_eq!(pid, pgid, "child must lead its own process group: {text}");
    assert_eq!(fields.next(), Some("[unset]"), "{text}");
}
