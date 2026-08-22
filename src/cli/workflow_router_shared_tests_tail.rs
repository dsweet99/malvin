use super::*;
pub(crate) fn artifact_storage_available() -> bool {
    let root = crate::malvin_home_logs_root();
    if std::fs::create_dir_all(&root).is_err() {
        return false;
    }
    let probe = root.join(format!(".test-write-probe-{}", std::process::id()));
    let available = std::fs::write(&probe, []).is_ok();
    let _ = std::fs::remove_file(probe);
    available
}

#[macro_export]
macro_rules! router_workflow_context_with_gates {
    ($artifacts:expr, $opts:expr, $include:expr) => {{
        let mut context =
            $crate::orchestrator::workflow_context_paths_only($artifacts, $opts.model, $opts.git);
        if $include {
            context.insert(
                "quality_gates".to_string(),
                $crate::repo_gates::prompt_quality_gates_markdown_ephemeral(&$artifacts.work_dir)
                    .expect("quality gates"),
            );
        }
        Ok::<_, String>(context)
    }};
}

#[macro_export]
macro_rules! router_workflow_context {
    ($artifacts:expr, $model:expr, $git:expr $(,)?) => {
        $crate::router_workflow_context_with_gates!(
            $artifacts,
            $crate::workflow_context::PromptModelOpts::new($model, $git),
            true
        )
    };
}

#[macro_export]
macro_rules! router_workflow_context_without_gates {
    ($artifacts:expr, $model:expr, $git:expr $(,)?) => {
        $crate::router_workflow_context_with_gates!(
            $artifacts,
            $crate::workflow_context::PromptModelOpts::new($model, $git),
            false
        )
    };
}

#[macro_export]
macro_rules! write_checks_do_not_pass_for_artifacts {
    ($artifacts:expr) => {{
        let path = $artifacts.artifact_review_md();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("review parent");
        }
        std::fs::write(path, b"Checks do not pass\n").expect("review marker");
        Ok::<(), String>(())
    }};
}

#[macro_export]
macro_rules! gate_iteration_context {
    ($base:expr, $artifacts:expr, $path:expr, $iteration:expr) => {{
        let mut ctx = $base.clone();
        ctx.insert(
            "exp_log".to_string(),
            $crate::format_prompt_path($path, &$artifacts.work_dir),
        );
        ctx.insert(
            "current_state".to_string(),
            $crate::current_state::format_current_state(
                $artifacts.work_dir.as_path(),
                Some($iteration),
                Some($artifacts),
            ),
        );
        ctx
    }};
}

#[test]
fn prefer_gate_outcome_surfaces_restore_when_gate_passed() {
    let err = prefer_gate_outcome_over_post_gate_cleanup(
        Ok(()),
        Err("malvin_checks restore: boom".into()),
    )
    .unwrap_err();
    assert!(err.contains("malvin_checks restore"));
}

#[test]
fn passing_gate_run_sets_just_ran_and_clear_resets_it() {
    crate::test_utils::with_isolated_home(|_| {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (_bin, _guard) = crate::test_agent_client::write_fake_gate(tmp.path(), "true", 0);
        let (artifacts, backups) = router_gates_restore_fixture(tmp.path());
        // Parallel tests share the process-global flag; only assert the
        // transition caused by this test's own gate run.
        run_router_workspace_gates(&artifacts, &backups, true).expect("gates pass");
        assert!(
            crate::gate_loop_session::quality_gates_just_ran(),
            "completed gate run must set just_ran"
        );

        // Clearing for the next agent turn resets it.
        clear_quality_gates_log_for_next_agent(&artifacts).expect("clear");
        assert!(
            !crate::gate_loop_session::quality_gates_just_ran(),
            "clearing the quality gates log must reset just_ran"
        );
        assert_eq!(
            std::fs::read_to_string(artifacts.quality_gates_log_path()).expect("read"),
            ""
        );
    });
}

pub(crate) fn router_gates_restore_fixture(
    work: &std::path::Path,
) -> (
    crate::artifacts::RunArtifacts,
    crate::artifacts::SessionDotfileBackups,
) {
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/gates"), "true\n").expect("gates");
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("code", Some(work)).expect("artifacts");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot");
    (artifacts, backups)
}
pub(crate) fn gitignore_restore_failure_fixture(
    work: &std::path::Path,
) -> (
    crate::artifacts::RunArtifacts,
    crate::artifacts::SessionDotfileBackups,
) {
    std::fs::create_dir_all(work.join(".malvin")).expect("mkdir");
    std::fs::write(work.join(".malvin/gates"), "true\n").expect("gates");
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("code", Some(work)).expect("artifacts");
    std::fs::write(work.join(".gitignore"), "orig\n").expect("gitignore");
    let backups = crate::artifacts::SessionDotfileBackups::snapshot(work).expect("snapshot");
    std::fs::remove_file(work.join(".gitignore")).expect("remove gitignore");
    std::fs::create_dir(work.join(".gitignore")).expect("gitignore dir");
    (artifacts, backups)
}
