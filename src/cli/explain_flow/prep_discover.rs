use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{explain_pdf_path_from_tex, ExplainPreflightSnapshot, ExplainResolvedOutputs};

pub(crate) fn resolve_explain_search_dir(request_work_dir: &Path, cwd: &Path) -> PathBuf {
    if request_work_dir.as_os_str() == "." {
        cwd.to_path_buf()
    } else if request_work_dir.is_absolute() {
        request_work_dir.to_path_buf()
    } else {
        cwd.join(request_work_dir)
    }
}

pub(crate) fn snapshot_tex_pdf_in_dir(dir: &Path) -> Result<HashSet<PathBuf>, String> {
    let mut paths = HashSet::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if path.is_file() && (ext == "tex" || ext == "pdf") {
            paths.insert(path);
        }
    }
    Ok(paths)
}

fn is_non_empty_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .ok()
        .is_some_and(|meta| meta.is_file() && meta.len() > 0)
}

fn explain_output_candidate(
    tex_path: PathBuf,
    snapshot: &ExplainPreflightSnapshot,
) -> Option<(SystemTime, PathBuf, PathBuf)> {
    if snapshot.pre_existing_tex_pdf.contains(&tex_path) {
        return None;
    }
    let pdf_path = explain_pdf_path_from_tex(&tex_path);
    if !is_non_empty_file(&tex_path) || !is_non_empty_file(&pdf_path) {
        return None;
    }
    let modified = std::fs::metadata(&tex_path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    Some((modified, tex_path, pdf_path))
}

fn collect_explain_output_candidates(
    search_dir: &Path,
    snapshot: &ExplainPreflightSnapshot,
) -> Result<Vec<(SystemTime, PathBuf, PathBuf)>, String> {
    let entries = std::fs::read_dir(search_dir).map_err(|e| {
        format!(
            "malvin explain: failed to scan `{}` for agent output: {e}",
            search_dir.display()
        )
    })?;
    Ok(entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("tex"))
        .filter_map(|tex_path| explain_output_candidate(tex_path, snapshot))
        .collect())
}

/// After an auto-named explain session, find the newest non-empty `.tex`/`.pdf` pair the agent wrote.
pub(crate) fn discover_explain_outputs_in_work_dir(
    request_work_dir: &Path,
    snapshot: &ExplainPreflightSnapshot,
) -> Result<ExplainResolvedOutputs, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let search_dir = resolve_explain_search_dir(request_work_dir, &cwd);
    let mut candidates = collect_explain_output_candidates(&search_dir, snapshot)?;
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    let Some((_, tex_path, pdf_path)) = candidates.into_iter().next() else {
        return Err(format!(
            "malvin explain: expected agent to write a non-empty `.tex`/`.pdf` pair in `{}`",
            search_dir.display()
        ));
    };
    Ok(ExplainResolvedOutputs { tex_path, pdf_path })
}
