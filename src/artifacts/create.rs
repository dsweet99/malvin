use std::path::{Path, PathBuf};

use super::RunArtifacts;

pub fn create_run_artifacts(
    plan_source: &Path,
    base_dir: Option<&Path>,
) -> std::io::Result<RunArtifacts> {
    create_run_artifacts_opts(plan_source, base_dir, crate::run_id::RunDirOptions::default())
}

pub fn create_run_artifacts_opts(
    plan_source: &Path,
    base_dir: Option<&Path>,
    opts: crate::run_id::RunDirOptions,
) -> std::io::Result<RunArtifacts> {
    let run_dir = crate::run_id::create_run_dir(base_dir, opts)?;
    let plan_target = run_dir.join("plan.md");
    std::fs::copy(plan_source, &plan_target)?;
    let artifacts = RunArtifacts {
        run_dir,
        plan_path: plan_target,
        work_dir: plan_source
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
    };
    #[cfg(not(test))]
    crate::stdout_log_path::set_stdout_log_path(Some(artifacts.stdout_log_path()));
    Ok(artifacts)
}

pub fn create_run_artifacts_from_text(
    plan_text: &str,
    base_dir: Option<&Path>,
) -> std::io::Result<RunArtifacts> {
    create_run_artifacts_from_text_opts(
        plan_text,
        base_dir,
        crate::run_id::RunDirOptions::default(),
    )
}

pub fn create_run_artifacts_from_text_opts(
    plan_text: &str,
    base_dir: Option<&Path>,
    opts: crate::run_id::RunDirOptions,
) -> std::io::Result<RunArtifacts> {
    let work_dir = base_dir.unwrap_or_else(|| Path::new(".")).to_path_buf();
    let run_dir = crate::run_id::create_run_dir(base_dir, opts)?;
    let plan_target = run_dir.join("plan.md");
    std::fs::write(&plan_target, plan_text)?;
    let artifacts = RunArtifacts {
        run_dir,
        plan_path: plan_target,
        work_dir,
    };
    #[cfg(not(test))]
    crate::stdout_log_path::set_stdout_log_path(Some(artifacts.stdout_log_path()));
    Ok(artifacts)
}

pub fn create_kpop_run_artifacts(
    request_text: &str,
    base_dir: Option<&Path>,
) -> std::io::Result<RunArtifacts> {
    create_kpop_run_artifacts_opts(
        request_text,
        base_dir,
        crate::run_id::RunDirOptions::default(),
    )
}

pub fn create_kpop_run_artifacts_opts(
    request_text: &str,
    base_dir: Option<&Path>,
    opts: crate::run_id::RunDirOptions,
) -> std::io::Result<RunArtifacts> {
    let work_dir = base_dir.unwrap_or_else(|| Path::new(".")).to_path_buf();
    let run_dir = crate::run_id::create_run_dir(base_dir, opts)?;
    let request_target = run_dir.join("request.md");
    std::fs::write(&request_target, request_text)?;
    let artifacts = RunArtifacts {
        run_dir,
        plan_path: request_target,
        work_dir,
    };
    #[cfg(not(test))]
    crate::stdout_log_path::set_stdout_log_path(Some(artifacts.stdout_log_path()));
    Ok(artifacts)
}
