use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::acp::{
    AgentError, AuthError, PromptRpcCleanup, ReviewerPromptPair,
    agent_string_is_retriable,
};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::kpop_test_stubs::{CaptureWants, EchoPrompts, MtStubPrompts};
use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::orchestrator::{ReviewPromptLog, review_prompt_log_path};
use crate::output::{format_line, set_stdout_log_path, timestamp_now_string};
use crate::run_timing::finalize_and_emit_run_timing;

#[test]
fn smoke_learn_gate_and_time_format() {
    assert!(crate::should_run_learn_check(0, 0));
    assert!(!crate::should_run_learn_check(60_000, 1));
    assert!(!crate::time_format::timestamp_now_string().is_empty());
}

#[test]
fn smoke_agent_error_and_auth_error_display() {
    let ae = AgentError("agent msg".into());
    assert_eq!(format!("{ae}"), "agent msg");
    let auth = AuthError("auth msg".into());
    assert_eq!(format!("{auth}"), "auth msg");
}

#[test]
fn smoke_reviewer_prompt_pair_struct() {
    let cwd = std::path::Path::new(".");
    let review = std::path::Path::new("review.md");
    let log = std::path::Path::new("log");
    let _pair = ReviewerPromptPair {
        cwd,
        workspace_review_path: review,
        artifact_review_path: None,
        review_body: "body",
        review_who: "who",
        review_log: log,
        sync_workspace_review: true,
    };
}

#[tokio::test]
async fn smoke_prompt_rpc_cleanup_clear() {
    use std::sync::atomic::{AtomicBool, AtomicU64};
    let busy = Arc::new(AtomicBool::new(true));
    let trace_writer: Arc<tokio::sync::Mutex<Option<crate::acp::PromptTraceWriter>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    let prompt_rpc_id = Arc::new(AtomicU64::new(42));
    let cleanup = PromptRpcCleanup {
        busy: busy.clone(),
        trace_writer: trace_writer.clone(),
        prompt_rpc_id: prompt_rpc_id.clone(),
        idle_notify: None,
    };
    cleanup.clear_if_prompt_response(42).await;
    assert!(!busy.load(Ordering::SeqCst));
    assert_eq!(prompt_rpc_id.load(Ordering::SeqCst), 0);
}

#[test]
fn smoke_retriable_timeout_delimited_token() {
    assert!(agent_string_is_retriable("rpc error: timeout"));
}

#[test]
fn smoke_kpop_turn_prompts_and_stubs() {
    let tmp = kiss_test_tmp();
    let store = kiss_smoke_prompt_store(&tmp.path().join("prompts"));
    let ctx = kiss_smoke_render_context();
    let mut turn = KpopTurnPrompts {
        store: &store,
        base: &ctx,
        request_text: "req",
        prepend_rules_once: true,
    };
    let _ = turn.kpop_block(1, 0).expect("kpop block");
    let _ = turn.mbc2_pure().expect("mbc2");

    let mut mt = MtStubPrompts;
    let _ = mt.kpop_block(2, 1).unwrap();
    let _ = mt.mbc2_pure().unwrap();

    let mut echo = EchoPrompts;
    let _ = echo.kpop_block(3, 0).unwrap();
    let _ = echo.mbc2_pure().unwrap();

    let wants = Arc::new(std::sync::Mutex::new(Vec::<usize>::new()));
    let mut cap = CaptureWants::new(wants.clone());
    let _ = cap.kpop_block(4, 0).unwrap();
    let _ = cap.mbc2_pure().unwrap();
    assert_eq!(wants.lock().unwrap().as_slice(), &[4]);
}

#[test]
fn smoke_kpop_multiturn_stub_mt_variant() {
    let mut stub = KpopMultiturnPrompts::StubMt(MtStubPrompts);
    assert!(stub.kpop_block(1, 0).unwrap().contains("stub"));
    assert!(stub.mbc2_pure().unwrap().contains("stub"));
}

#[test]
fn smoke_multiturn_prompt_enum() {
    match crate::multiturn_prompt::MultiturnPrompt::Mbc2("z".into()) {
        crate::multiturn_prompt::MultiturnPrompt::Mbc2(s) => assert_eq!(s, "z"),
        crate::multiturn_prompt::MultiturnPrompt::KpopBlock(_) => panic!("expected mbc2"),
    }
}

#[test]
fn smoke_run_id_create_run_dir() {
    let tmp = kiss_test_tmp();
    let run_dir = crate::run_id::create_run_dir(Some(tmp.path())).expect("run dir");
    assert!(run_dir.is_dir());
    assert!(run_dir.starts_with(tmp.path().join("_malvin")));
}

#[test]
fn smoke_output_timestamp_and_stdout_log() {
    assert!(!timestamp_now_string().is_empty());
    let line = format_line("malvin", "hello");
    assert!(line.contains("hello"));
    let tmp = kiss_test_tmp();
    let log = tmp.path().join("out.log");
    set_stdout_log_path(Some(log.clone()));
    crate::output::print_stdout_line("malvin", "logged");
    set_stdout_log_path(None);
    let body = std::fs::read_to_string(log).expect("log");
    assert!(body.contains("logged"));
}

#[test]
fn smoke_orchestrator_helper_paths() {
    let tmp = kiss_test_tmp();
    let art = crate::artifacts::create_run_artifacts_from_text("p", Some(tmp.path())).expect("art");
    let ctx = crate::orchestrator::workflow_context_paths_only(&art, "code");
    assert!(ctx.contains_key("plan_path"));
    let exp = art.exp_log_path();
    std::fs::create_dir_all(exp.parent().unwrap()).expect("dir");
    std::fs::write(&exp, "e").expect("exp");
    assert!(!crate::orchestrator::format_exp_log_relative(&art, &exp).is_empty());
}

#[test]
fn smoke_review_prompt_log_suffix() {
    let tmp = kiss_test_tmp();
    let art = crate::artifacts::create_run_artifacts_from_text("p", Some(tmp.path())).expect("art");
    let input = ReviewPromptLog {
        prompt_file: "review_write.md",
        skip_repo_style: false,
        log_attempt: 1,
        attempt: 2,
    };
    let path = review_prompt_log_path(&art, input);
    assert!(path.to_string_lossy().contains("try_2"));
}

#[test]
fn smoke_run_timing_finalize_paths() {
    let tmp = kiss_test_tmp();
    let timing = crate::run_timing::RunTiming::new_arc();
    {
        let mut g = timing.lock().unwrap();
        g.mark_wall_start(std::time::Instant::now());
        g.mark_wall_end(std::time::Instant::now());
    }
    finalize_and_emit_run_timing(tmp.path(), &timing).expect("emit");
    crate::run_timing::finalize_run_timing_json_only(tmp.path(), &timing).expect("json only");
}

#[cfg(target_os = "linux")]
#[test]
fn smoke_child_health_linux_sample() {
    let h = crate::child_health::sample_child_health(1);
    let _ = (h.exists, h.zombie, h.counters_trusted);
}

#[cfg(target_os = "macos")]
#[test]
fn smoke_child_health_macos_sample() {
    let _ = crate::child_health::sample_child_health(1);
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn smoke_child_health_other_sample() {
    let h = crate::child_health::other::sample_child_health_other(1);
    assert!(h.exists);
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn smoke_linux_verify_failed_abort_path() {
    use crate::acp_memory_containment::{
        complete_and_require_linux_containment_after_spawn, ContainmentHandle,
    };
    let mut child = tokio::process::Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("spawn sleep");
    let err = complete_and_require_linux_containment_after_spawn(
        child.id(),
        ContainmentHandle::Inactive,
        true,
        &mut child,
    )
    .await
    .expect_err("inactive containment");
    assert!(err.contains("verify failed"));
    assert!(child.try_wait().expect("wait").is_some());
}
