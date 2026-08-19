use super::create_run_artifacts_from_text_opts;

fn seed_home_logs_for_gc_test(work_dir: &std::path::Path) -> std::path::PathBuf {
    let logs = crate::malvin_logs_root(work_dir);
    std::fs::create_dir_all(&logs).unwrap();
    for name in [
        "20260101_000000_aaaaaaa1",
        "20260102_000000_bbbbbbb2",
        "20260103_000000_ccccccc3",
    ] {
        std::fs::create_dir_all(logs.join(name)).unwrap();
        std::fs::write(logs.join(name).join("payload"), vec![0u8; 500]).unwrap();
    }
    let config_path = crate::malvin_config_path(work_dir);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(
        &config_path,
        "[logs]\nmax_age_days = 0\nmax_bytes = \"1000B\"\n",
    )
    .unwrap();
    logs
}

#[test]
fn create_run_artifacts_from_text_prunes_old_runs_after_new_dir() {
    crate::test_utils::with_isolated_home(|work| {
        let logs = seed_home_logs_for_gc_test(work);
        let art = create_run_artifacts_from_text_opts(
            "prompt",
            Some(work),
            crate::run_id::RunDirOptions { gc: true },
        )
        .unwrap();
        assert!(!logs.join("20260101_000000_aaaaaaa1").exists());
        assert!(logs.join("20260102_000000_bbbbbbb2").exists());
        assert!(logs.join("20260103_000000_ccccccc3").exists());
        assert!(art.run_dir.starts_with(&logs));
        assert!(
            art.run_dir.is_dir(),
            "active run must survive post-create prune"
        );
    });
}

#[test]
fn command_log_can_be_written_before_deferred_prune() {
    crate::test_utils::with_isolated_home(|work| {
        let logs = seed_home_logs_for_gc_test(work);
        let art = create_run_artifacts_from_text_opts(
            "prompt",
            Some(work),
            crate::run_id::RunDirOptions { gc: false },
        )
        .unwrap();
        assert!(
            logs.join("20260101_000000_aaaaaaa1").exists(),
            "deferred GC must not prune during create"
        );
        crate::cli::run_emit::emit_command_line(&art.run_dir, false).expect("command.log");
        let command_log = std::fs::read_to_string(art.run_dir.join("command.log")).expect("read");
        assert!(command_log.contains("Command:"));
        crate::log_gc::prune_logs_after_run_created(work, &art.run_dir);
        assert!(!logs.join("20260101_000000_aaaaaaa1").exists());
        assert!(art.run_dir.is_dir());
        assert!(
            art.run_dir.join("command.log").is_file(),
            "Command: log must remain after prune"
        );
    });
}
