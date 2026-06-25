use super::*;

#[test]
fn collect_recent_delight_plans_empty_when_no_logs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("plan.md");
    assert!(collect_recent_delight_plan_paths(tmp.path(), &out).is_empty());
}

#[test]
fn collect_recent_delight_plans_finds_prior_out_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("old.md"), "x\n").expect("write");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp.path());
    let run_dir = logs_root.join("20260101_120000_abc12345");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    std::fs::write(
        run_dir.join("command.log"),
        "Command: malvin delight --out-path old.md\n",
    )
    .expect("log");
    let out = tmp.path().join("plan.md");
    let paths = collect_recent_delight_plan_paths(tmp.path(), &out);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("old.md"));
}

#[test]
fn collect_recent_delight_plans_defaults_to_plan_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("plan.md"), "prior\n").expect("write");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp.path());
    let run_dir = logs_root.join("20260102_120000_abc12345");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    std::fs::write(run_dir.join("command.log"), "Command: malvin delight\n").expect("log");
    let out = tmp.path().join("new.md");
    let paths = collect_recent_delight_plan_paths(tmp.path(), &out);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("plan.md"));
}

#[test]
fn collect_recent_delight_plans_skips_missing_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp.path());
    let run_dir = logs_root.join("20260101_120000_abc12345");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    std::fs::write(
        run_dir.join("command.log"),
        "Command: malvin delight --out-path gone.md\n",
    )
    .expect("log");
    let out = tmp.path().join("plan.md");
    assert!(collect_recent_delight_plan_paths(tmp.path(), &out).is_empty());
}

#[test]
fn collect_recent_delight_plans_caps_at_five() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp.path());
    for i in 0..6 {
        std::fs::write(tmp.path().join(format!("p{i}.md")), "x\n").expect("write");
        let run_dir = logs_root.join(format!("2026010{i}_120000_abc1234{i}"));
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(
            run_dir.join("command.log"),
            format!("Command: malvin delight --out-path p{i}.md\n"),
        )
        .expect("log");
    }
    let out = tmp.path().join("new.md");
    assert_eq!(collect_recent_delight_plan_paths(tmp.path(), &out).len(), 5);
}

#[test]
fn collect_recent_delight_plans_dedupes_repeated_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("plan.md"), "prior\n").expect("write");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp.path());
    for run in ["20260101_120000_abc12345", "20260102_120000_abc12346"] {
        let run_dir = logs_root.join(run);
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(run_dir.join("command.log"), "Command: malvin delight\n").expect("log");
    }
    let out = tmp.path().join("plan_1.md");
    let paths = collect_recent_delight_plan_paths(tmp.path(), &out);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("plan.md"));
}

#[test]
fn collect_recent_delight_plans_excludes_current_out_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("plan.md"), "x\n").expect("write");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp.path());
    let run_dir = logs_root.join("20260101_120000_abc12345");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    std::fs::write(run_dir.join("command.log"), "Command: malvin delight\n").expect("log");
    let paths = collect_recent_delight_plan_paths(tmp.path(), &tmp.path().join("plan.md"));
    assert!(paths.is_empty());
}
