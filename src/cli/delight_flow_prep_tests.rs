use crate::cli::WorkflowCliOptions;
use crate::prompts::PromptStore;

use super::*;

fn seed_prior_delight_pitch(tmp: &std::path::Path, out_rel: &str) {
    std::fs::write(tmp.join(out_rel), "old pitch\n").expect("write old");
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
    guidance: Option<&str>,
) -> String {
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("delight", Some(tmp)).expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    delight_kpop_request(&store, &artifacts, out, guidance).expect("request")
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
    let out = tmp.path().join("pitch.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out, None);
    assert!(
        !text.contains("{{"),
        "delight kpop request must expand all placeholders: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_out_pitch_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("plans/delight.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out, None);
    assert!(
        text.contains("plans/delight.md") || text.contains("./plans/delight.md"),
        "expected out_pitch_path in request: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_kpop_program_wrapper() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("pitch.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out, None);
    assert!(
        text.contains("Satisfy all constraints"),
        "expected kpop_program wrapper: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_recent_delight_pitches_when_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    seed_prior_delight_pitch(tmp.path(), "old.md");
    let out = tmp.path().join("pitch.md");
    let artifacts = crate::artifacts::create_kpop_run_artifacts_opts(
        "delight",
        Some(tmp.path()),
        crate::run_id::RunDirOptions::without_gc(),
    )
    .expect("artifacts");
    let store = PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let text = delight_kpop_request(&store, &artifacts, &out, None).expect("request");
    assert!(
        text.contains("old.md"),
        "expected recent delight pitch path in request: {text:?}"
    );
}

#[test]
fn delight_kpop_request_empty_recent_delight_pitches_when_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("pitch.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out, None);
    assert!(
        !text.contains("{{ recent_delight_pitchs }}"),
        "placeholder must be expanded: {text:?}"
    );
}

#[test]
fn delight_kpop_request_includes_guidance_when_provided() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("pitch.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out, Some("focus on CLI UX"));
    assert!(
        text.contains("focus on CLI UX"),
        "expected guidance in request: {text:?}"
    );
    assert!(
        text.contains("Follow this user guidance"),
        "expected guidance header: {text:?}"
    );
}

#[test]
fn delight_kpop_request_omits_guidance_block_when_none() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = tmp.path().join("pitch.md");
    let text = delight_kpop_request_in_workspace(tmp.path(), &out, None);
    assert!(
        !text.contains("Follow this user guidance"),
        "guidance block must be absent when unset: {text:?}"
    );
}

#[test]
fn resolve_delight_guidance_reads_md_file() {
    crate::test_utils::with_isolated_home(|work| {
        std::fs::write(work.join("hint.md"), "from file\n").expect("write");
        std::env::set_current_dir(work).expect("chdir");
        let got = resolve_delight_guidance(Some(&"hint.md".to_string())).expect("resolve");
        assert_eq!(got.as_deref(), Some("from file\n"));
    });
}

#[test]
fn resolve_delight_guidance_none_for_empty_string() {
    assert!(resolve_delight_guidance(Some(&String::new()))
        .expect("resolve")
        .is_none());
}

#[test]
fn format_delight_guidance_block_empty_when_blank() {
    assert!(format_delight_guidance_block(None).is_empty());
    assert!(format_delight_guidance_block(Some("   ")).is_empty());
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
        "pitch.md"
    );
}
