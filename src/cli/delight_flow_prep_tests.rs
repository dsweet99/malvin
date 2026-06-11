use crate::cli::WorkflowCliOptions;
use crate::prompts::PromptStore;

use super::*;

fn seed_prior_delight_plan(tmp: &std::path::Path, out_rel: &str) {
    std::fs::write(tmp.join(out_rel), "old plan\n").expect("write old");
    let logs_root = crate::workspace_paths::malvin_logs_root(tmp);
    let run_dir = logs_root.join("20260101_120000_abc12345");
    std::fs::create_dir_all(&run_dir).expect("mkdir run");
    std::fs::write(
        run_dir.join("command.log"),
        format!("Command: malvin delight --out-path {out_rel}\n"),
    )
    .expect("write command.log");
}

fn delight_kpop_request_in_workspace(
    tmp: &std::path::Path,
    out: &std::path::Path,
) -> String {
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("delight", Some(tmp)).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    delight_kpop_request(&store, &artifacts, out).expect("request")
}

#[test]
fn default_constraints_prompt_embeds_delight() {
    assert!(crate::prompts::default_file("delight_constraints.md").is_some());
}

#[test]
fn default_prompts_list_includes_delight_constraints() {
    assert!(crate::prompts::DEFAULT_PROMPTS.contains(&"delight_constraints.md"));
}

#[test]
fn prepare_delight_kpop_prompt_store_loads_program_and_constraints() {
    let workflow = WorkflowCliOptions { force: false };
    let store = prepare_delight_kpop_prompt_store(workflow).expect("store");
    assert!(store.validate_exists("kpop_program.md").is_ok());
    assert!(store.validate_exists("delight_constraints.md").is_ok());
}

#[test]
fn delight_kpop_request_has_no_unresolved_braces() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("plan.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        !text.contains("{{"),
        "delight kpop request must expand all placeholders: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_out_plan_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("plans/delight.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        text.contains("plans/delight.md") || text.contains("./plans/delight.md"),
        "expected out_plan_path in request: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_kpop_program_wrapper() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("plan.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        text.contains("Satisfy all constraints"),
        "expected kpop_program wrapper: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_recent_delight_plans_when_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_prior_delight_plan(tmp.path(), "old.md");
    let out = tmp.path().join("plan.md");
    let artifacts = crate::artifacts::create_kpop_run_artifacts_opts(
        "delight",
        Some(tmp.path()),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let text = delight_kpop_request(&store, &artifacts, &out).expect("request");
    assert!(
        text.contains("old.md"),
        "expected recent delight plan path in request: {text:?}"
    );
}

#[test]
fn delight_kpop_request_empty_recent_delight_plans_when_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("plan.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out);
    assert!(
        !text.contains("{{ recent_delight_plans }}"),
        "placeholder must be expanded: {text:?}"
    );
}

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

#[test]
fn parse_delight_out_path_from_command_line_variants() {
    assert_eq!(
        parse_delight_out_path_from_command_line("Command: malvin delight --out-path plans/x.md"),
        "plans/x.md"
    );
    assert_eq!(
        parse_delight_out_path_from_command_line("Command: malvin delight --out-path=plans/x.md"),
        "plans/x.md"
    );
    assert_eq!(
        parse_delight_out_path_from_command_line("Command: malvin delight"),
        "plan.md"
    );
}
