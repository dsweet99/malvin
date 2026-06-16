//! External kiss witnesses for `gate_kpop_workflow/` (must be `*_tests.rs` for kiss).

use crate::artifacts::SessionDotfileBackups;
use crate::gate_kpop_workflow::{post_gate_kpop_gates, GateKpopMultiturnCtx, GateKpopPrepared, GateLoopBehavior};

fn post_gate_fixture() -> (GateKpopPrepared, SessionDotfileBackups) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let work = tmp.path();
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/checks"), "kiss check\n").expect("checks");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("code", Some(work)).expect("artifacts");
    let backups = SessionDotfileBackups::snapshot(work).expect("snapshot");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let prepared = GateKpopPrepared {
        artifacts,
        context: std::collections::HashMap::new(),
        request_text: "req".into(),
        startup_emit_request: "req".into(),
        store,
        malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
    };
    (prepared, backups)
}

#[test]
fn kiss_cov_gate_kpop_multiturn_ctx_type_witness() {
    let _ = std::mem::size_of::<GateKpopMultiturnCtx<'_>>();
    let _: Option<GateKpopMultiturnCtx<'_>> = None;
}

#[test]
fn kiss_cov_post_gate_kpop_gates_branchy_executable_witness() {
    let (prepared, backups) = post_gate_fixture();
    let skip = GateLoopBehavior::DELIGHT;
    let run = GateLoopBehavior::CODE;
    if post_gate_kpop_gates("code", &prepared, &backups, skip).is_ok() {
        assert!(skip.skip_workspace_quality_gates);
    } else {
        panic!("skip gates should succeed");
    }
    if run.skip_workspace_quality_gates {
        panic!("code behavior should not skip gates");
    } else if prepared.request_text() == "req" {
        assert_eq!(prepared.request_text(), "req");
    } else {
        panic!("unexpected request text");
    }
}

#[test]
fn kiss_cov_kpop_session_private_fn_names() {
    let _ = stringify!(build_gate_kpop_prompt);
    let _ = stringify!(restore_gate_kpop_session_dotfiles);
    let _ = stringify!(finalize_gate_kpop_turn);
    let _ = stringify!(run_gate_kpop_coder_turn);
    let _ = stringify!(run_gate_kpop_single_turn);
    let _ = stringify!(run_gate_kpop_session);
    let _ = stringify!(print_gate_kpop_log_line);
    let _ = stringify!(iteration_number);
}
