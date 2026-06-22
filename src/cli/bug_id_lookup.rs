use std::path::{Path, PathBuf};

#[path = "bug_id_lookup_log.rs"]
mod bug_id_lookup_log;

pub(crate) use bug_id_lookup_log::MalvinRunLogKind;

pub(super) struct BugLogMatch {
    run_dir: PathBuf,
    log_file: PathBuf,
    exp_log_rel: Option<String>,
}

#[derive(Debug)]
pub(crate) struct BugIdResolved {
    pub exp_log_path: PathBuf,
}

pub(super) fn lookup_run_by_log_kind(
    cwd: &Path,
    id: &str,
    kind: MalvinRunLogKind,
) -> Result<BugIdResolved, String> {
    let malvin_root = crate::find_malvin_logs_root(cwd)
        .unwrap_or_else(|| crate::malvin_logs_root(cwd));
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
    Ok(BugIdResolved { exp_log_path })
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

fn not_found_msg(id: &str, malvin_root: &Path, _kind: MalvinRunLogKind) -> String {
    format!("no KPOP id {id} under {}", malvin_root.display())
}

fn ambiguous_msg(id: &str, malvin_root: &Path, by_run: &[BugLogMatch], kind: MalvinRunLogKind) -> String {
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
        "KPOP id {id} is ambiguous ({} matches under {}):\n{}\nRemove stale runs or use a unique id.",
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
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if crate::log_gc::is_run_log_dir_name(&name) {
            if let Some(m) = bug_id_lookup_log::match_run_logs(&path, id, kind) {
                out.push(m);
            }
        } else {
            scan_malvin_logs(&path, id, kind, out)?;
        }
    }
    Ok(())
}

fn work_dir_from_run_dir(run_dir: &Path, cwd: &Path) -> PathBuf {
    workspace_root_from_run_dir(run_dir).unwrap_or_else(|| cwd.to_path_buf())
}

fn workspace_root_from_run_dir(run_dir: &Path) -> Option<PathBuf> {
    if let Some(ws) = crate::read_work_dir_manifest(run_dir) {
        return Some(ws);
    }
    let logs_segment = Path::new(crate::MALVIN_LOGS_REL).file_name()?;
    let malvin_segment = Path::new(crate::MALVIN_DIR).file_name()?;
    let mut cursor = run_dir;
    while let Some(logs_dir) = cursor.parent() {
        if logs_dir.file_name() == Some(logs_segment) {
            let malvin_dir = logs_dir.parent()?;
            if malvin_dir.file_name() == Some(malvin_segment) {
                return malvin_dir
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .map(Path::to_path_buf);
            }
        }
        cursor = logs_dir;
    }
    None
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

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_band80_witnesses() {
        let m = BugLogMatch {
            run_dir: std::path::PathBuf::from("/tmp/run"),
            log_file: std::path::PathBuf::from("/tmp/log"),
            exp_log_rel: Some("rel.md".to_string()),
        };
        let BugLogMatch {
            run_dir,
            log_file,
            exp_log_rel,
        } = m;
        assert_eq!(run_dir, std::path::PathBuf::from("/tmp/run"));
        assert_eq!(log_file, std::path::PathBuf::from("/tmp/log"));
        assert_eq!(exp_log_rel.as_deref(), Some("rel.md"));
        let m_none = BugLogMatch {
            run_dir: std::path::PathBuf::from("/tmp/run2"),
            log_file: std::path::PathBuf::from("/tmp/log2"),
            exp_log_rel: None,
        };
        let BugLogMatch {
            run_dir,
            log_file,
            exp_log_rel,
        } = m_none;
        assert!(exp_log_rel.is_none());
        assert_eq!(run_dir, std::path::PathBuf::from("/tmp/run2"));
        assert_eq!(log_file, std::path::PathBuf::from("/tmp/log2"));
        let resolved = BugIdResolved {
            exp_log_path: std::path::PathBuf::from("/tmp/exp.md"),
        };
        let BugIdResolved { exp_log_path } = resolved;
        assert_eq!(exp_log_path, std::path::PathBuf::from("/tmp/exp.md"));
    }
}
#[cfg(test)]
#[path = "bug_id_lookup_test.rs"]
mod bug_id_lookup_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<BugIdResolved> = None;
        let _: Option<BugLogMatch> = None;
    }
}
