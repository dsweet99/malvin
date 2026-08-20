use super::*;
pub(crate) fn artifact_storage_available() -> bool {
    let root = crate::malvin_home_logs_root();
    if std::fs::create_dir_all(&root).is_err() { return false; }
    let probe = root.join(format!(".test-write-probe-{}", std::process::id()));
    let available = std::fs::write(&probe, []).is_ok();
    let _ = std::fs::remove_file(probe);
    available
}

#[macro_export]
macro_rules! router_workflow_context_with_gates {
    ($artifacts:expr, $opts:expr, $include:expr) => {{
        let mut context = $crate::orchestrator::workflow_context_paths_only(
            $artifacts,
            $opts.model,
            $opts.git,
        );
        if $include {
            context.insert(
                "quality_gates".to_string(),
                $crate::repo_gates::prompt_quality_gates_markdown_ephemeral(
                    &$artifacts.work_dir,
                )
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
