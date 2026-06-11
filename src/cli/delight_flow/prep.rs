use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::log_gc::list_run_dirs;
use crate::prompts::{PromptError, PromptStore};
use crate::workflow_context::{format_prompt_path, insert_formatted};

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::default_output_path::{
    allocate_default_sibling_file, path_relative_to_cwd, DELIGHT_DEFAULT_OUT_PATH,
};
use crate::cli::workflow_kpop_shared::render_kpop_program_request_creative;

const DELIGHT_COMMAND_MARKER: &str = "Command: malvin delight";
const MAX_RECENT_DELIGHT_PLANS: usize = 5;

pub(crate) fn prepare_delight_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program_creative.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("delight_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

pub(crate) fn parse_delight_out_path_from_command_line(command_line: &str) -> String {
    let args: Vec<&str> = command_line.split_whitespace().collect();
    let Some(idx) = args.iter().position(|&a| a == "delight") else {
        return "plan.md".to_string();
    };
    let tail = &args[idx + 1..];
    let mut i = 0;
    while i < tail.len() {
        if tail[i] == "--out-path" {
            if let Some(path) = tail.get(i + 1) {
                return (*path).to_string();
            }
            break;
        }
        if let Some(rest) = tail[i].strip_prefix("--out-path=") {
            if !rest.is_empty() {
                return rest.to_string();
            }
        }
        i += 1;
    }
    "plan.md".to_string()
}

fn delight_out_rel_from_command_log(text: &str) -> Option<String> {
    if !text.contains(DELIGHT_COMMAND_MARKER) {
        return None;
    }
    Some(
        text.lines()
            .find_map(|line| {
                line.contains(DELIGHT_COMMAND_MARKER)
                    .then(|| parse_delight_out_path_from_command_line(line))
            })
            .unwrap_or_else(|| "plan.md".to_string()),
    )
}

fn delight_plan_candidate_from_run(
    run_dir: &Path,
    work_dir: &Path,
    resolved_out_path: &Path,
) -> Option<PathBuf> {
    let text = std::fs::read_to_string(run_dir.join("command.log")).ok()?;
    let out_rel = delight_out_rel_from_command_log(&text)?;
    let candidate = work_dir.join(out_rel);
    if candidate == resolved_out_path || !candidate.is_file() {
        return None;
    }
    Some(candidate)
}

pub(crate) fn collect_recent_delight_plan_paths(
    work_dir: &Path,
    resolved_out_path: &Path,
) -> Vec<PathBuf> {
    let logs_root = crate::workspace_paths::malvin_logs_root(work_dir);
    if !logs_root.is_dir() {
        return Vec::new();
    }
    let mut run_dirs = list_run_dirs(&logs_root);
    run_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let mut collected = Vec::new();
    for run_dir in run_dirs {
        if collected.len() >= MAX_RECENT_DELIGHT_PLANS {
            break;
        }
        if let Some(candidate) =
            delight_plan_candidate_from_run(&run_dir, work_dir, resolved_out_path)
        {
            if !collected.iter().any(|existing| existing == &candidate) {
                collected.push(candidate);
            }
        }
    }
    collected
}

fn format_recent_delight_plans(work_dir: &Path, paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    paths.iter().fold(String::new(), |mut acc, p| {
        use std::fmt::Write as _;
        let _ = writeln!(acc, "- {}", format_prompt_path(p, work_dir));
        acc
    })
}

pub(crate) fn delight_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    resolved_out_path: &Path,
) -> Result<String, String> {
    let workspace_root = artifacts.work_dir.as_path();
    let recent_paths = collect_recent_delight_plan_paths(workspace_root, resolved_out_path);
    let recent_delight_plans = format_recent_delight_plans(workspace_root, &recent_paths);
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "out_plan_path", resolved_out_path, workspace_root);
    ctx.insert("recent_delight_plans".to_string(), recent_delight_plans);
    render_kpop_program_request_creative(store, "delight_constraints.md", &ctx, artifacts)
}

pub(crate) fn delight_preflight(out_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let resolved_out_path = if out_path == DELIGHT_DEFAULT_OUT_PATH {
        let default = cwd.join(DELIGHT_DEFAULT_OUT_PATH);
        allocate_default_sibling_file(&default, "plan", ".md")?
    } else {
        let resolved = cwd.join(out_path);
        if resolved.exists() {
            return Err(format!(
                "malvin delight: `{}` already exists; refusing to overwrite",
                resolved.display()
            ));
        }
        resolved
    };
    if let Some(parent) = resolved_out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let rel_out = path_relative_to_cwd(&resolved_out_path)?;
    let work_dir = crate::artifacts::work_dir_for_path(Path::new(&rel_out));
    Ok((resolved_out_path, work_dir))
}

#[cfg(test)]
#[path = "../delight_flow_prep_tests.rs"]
mod delight_flow_prep_tests;

#[cfg(test)]
#[path = "../delight_flow_prep_preflight_tests.rs"]
mod delight_flow_prep_preflight_tests;
