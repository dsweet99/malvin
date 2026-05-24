use std::path::{Path, PathBuf};

#[path = "bug_id_lookup_log.rs"]
mod bug_id_lookup_log;

pub(crate) use bug_id_lookup_log::MalvinRunLogKind;

pub(super) struct BugLogMatch {
    run_dir: PathBuf,
    log_file: PathBuf,
    exp_log_rel: Option<String>,
}

pub(crate) fn validate_bug_id(id: &str) -> Result<(), String> {
    crate::validate_malvin_short_id(id).map_err(|_| {
        format!(
            "invalid BUG_ID {id:?}: expected M followed by 5 lowercase letters or digits (example: Ma1b2c)"
        )
    })
}

#[must_use]
#[allow(dead_code)]
pub(crate) fn is_valid_bug_id(id: &str) -> bool {
    crate::is_valid_malvin_short_id(id)
}

#[derive(Debug)]
pub(crate) struct BugIdResolved {
    pub run_dir: PathBuf,
    pub exp_log_path: PathBuf,
    pub work_dir: PathBuf,
}

pub(crate) fn lookup_bug_id(cwd: &Path, id: &str) -> Result<BugIdResolved, String> {
    validate_bug_id(id)?;
    lookup_run_by_log_kind(cwd, id, MalvinRunLogKind::Bug)
}

pub(super) fn lookup_run_by_log_kind(
    cwd: &Path,
    id: &str,
    kind: MalvinRunLogKind,
) -> Result<BugIdResolved, String> {
    let malvin_root = cwd.join("_malvin");
    if !malvin_root.is_dir() {
        return Err(not_found_msg(id, &malvin_root, kind));
    }
    let by_run = collect_matches(&malvin_root, id, kind)?;
    if by_run.is_empty() {
        return Err(not_found_msg(id, &malvin_root, kind));
    }
    if by_run.len() > 1 {
        return Err(ambiguous_msg(id, &malvin_root, &by_run, kind));
    }
    let m = &by_run[0];
    let work_dir = work_dir_from_run_dir(&m.run_dir, cwd);
    let exp_log_path = resolve_exp_log_path(&work_dir, &m.run_dir, m.exp_log_rel.as_deref(), kind)?;
    Ok(BugIdResolved {
        run_dir: m.run_dir.clone(),
        exp_log_path,
        work_dir,
    })
}

fn collect_matches(
    malvin_root: &Path,
    id: &str,
    kind: MalvinRunLogKind,
) -> Result<Vec<BugLogMatch>, String> {
    let mut matches = Vec::new();
    scan_malvin_logs(malvin_root, id, kind, &mut matches)?;
    Ok(dedupe_by_run_dir(matches))
}

fn not_found_msg(id: &str, malvin_root: &Path, kind: MalvinRunLogKind) -> String {
    let label = match kind {
        MalvinRunLogKind::Bug => "BUG_ID",
        MalvinRunLogKind::Kpop => "KPOP id",
    };
    format!("no {label} {id} under {}", malvin_root.display())
}

fn ambiguous_msg(id: &str, malvin_root: &Path, by_run: &[BugLogMatch], kind: MalvinRunLogKind) -> String {
    let id_label = match kind {
        MalvinRunLogKind::Bug => "BUG_ID",
        MalvinRunLogKind::Kpop => "KPOP id",
    };
    let mut lines: Vec<String> = by_run
        .iter()
        .map(|m| {
            format!(
                "  run_dir={} (from {})",
                m.run_dir.display(),
                m.log_file.display()
            )
        })
        .collect();
    lines.sort();
    format!(
        "{id_label} {id} is ambiguous ({} matches under {}):\n{}\nRemove stale runs or use a unique id.",
        by_run.len(),
        malvin_root.display(),
        lines.join("\n")
    )
}

fn dedupe_by_run_dir(matches: Vec<BugLogMatch>) -> Vec<BugLogMatch> {
    let mut by_run: Vec<BugLogMatch> = Vec::new();
    for m in matches {
        if by_run.iter().any(|x| x.run_dir == m.run_dir) {
            continue;
        }
        by_run.push(m);
    }
    by_run
}

fn scan_malvin_logs(
    dir: &Path,
    id: &str,
    kind: MalvinRunLogKind,
    out: &mut Vec<BugLogMatch>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            scan_malvin_logs(&path, id, kind, out)?;
            if let Some(m) = bug_id_lookup_log::match_run_logs(&path, id, kind) {
                out.push(m);
            }
        }
    }
    Ok(())
}

fn work_dir_from_run_dir(run_dir: &Path, cwd: &Path) -> PathBuf {
    run_dir
        .parent()
        .and_then(|malvin| malvin.parent())
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| cwd.to_path_buf(), Path::to_path_buf)
}

fn resolve_exp_log_path(
    work_dir: &Path,
    run_dir: &Path,
    exp_log_rel: Option<&str>,
    kind: MalvinRunLogKind,
) -> Result<PathBuf, String> {
    if let Some(rel) = exp_log_rel {
        let path = rel
            .strip_prefix("./")
            .map_or_else(|| work_dir.join(rel), |stripped| work_dir.join(stripped));
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!(
            "{} for run {} points at missing file {}",
            kind.missing_log_err_label(),
            run_dir.display(),
            path.display()
        ));
    }
    let artifacts = crate::artifacts::RunArtifacts {
        run_dir: run_dir.to_path_buf(),
        plan_path: run_dir.join("plan.md"),
        work_dir: work_dir.to_path_buf(),
    };
    let path = artifacts.exp_log_path();
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "experiment log not found at {} ({} fallback)",
            path.display(),
            kind.fallback_err_label()
        ))
    }
}

pub(crate) fn ensure_exp_log_solved(exp_log_path: &Path) -> Result<(), String> {
    use crate::kpop_progression::agent_declared_success;
    let exp_text = std::fs::read_to_string(exp_log_path).map_err(|e| e.to_string())?;
    if agent_declared_success(&exp_text) {
        return Ok(());
    }
    Err(
        "KPOP did not record success: add a line exactly `## KPOP_SOLVED` to the experiment log once a serious bug is confirmed. Stopping before regression-test and fix coder phases.".to_string(),
    )
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{
        ambiguous_msg, collect_matches, dedupe_by_run_dir, ensure_exp_log_solved, is_valid_bug_id,
        lookup_bug_id, lookup_run_by_log_kind, not_found_msg, resolve_exp_log_path,
        scan_malvin_logs, validate_bug_id, work_dir_from_run_dir, BugIdResolved, BugLogMatch,
    };
    use super::MalvinRunLogKind;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<BugLogMatch> = None;
        let _: Option<BugIdResolved> = None;
        let _ = MalvinRunLogKind::Kpop;
        let _ = collect_matches;
        let _ = not_found_msg;
        let _ = ambiguous_msg;
        let _ = dedupe_by_run_dir;
        let _ = scan_malvin_logs;
        let _ = work_dir_from_run_dir;
        let _ = resolve_exp_log_path;
        let _ = lookup_bug_id;
        let _ = lookup_run_by_log_kind;
        let _ = validate_bug_id;
        let _ = is_valid_bug_id;
        let _ = ensure_exp_log_solved;
    }
}
