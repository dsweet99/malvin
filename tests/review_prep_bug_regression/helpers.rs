use std::path::{Path, PathBuf};
use std::process::Command;

pub fn manifest_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn assert_tracked_in_git(rel: &str) {
    let path = manifest_root().join(rel);
    assert!(
        path.is_file(),
        "bug: expected source file on disk: {}",
        path.display()
    );
    let out = Command::new("git")
        .args(["ls-files", "--error-unmatch", rel])
        .current_dir(manifest_root())
        .output()
        .expect("git ls-files");
    assert!(
        out.status.success(),
        "bug: {rel} is wired in the crate but not tracked in git (git ls-files --error-unmatch)"
    );
}

fn rel_path_from_mod_file(mod_file: &Path, path_attr: &str) -> PathBuf {
    mod_file
        .parent()
        .expect("mod parent")
        .join(path_attr)
        .canonicalize()
        .unwrap_or_else(|_| mod_file.parent().expect("mod parent").join(path_attr))
}

fn path_attrs_in(mod_content: &str) -> Vec<String> {
    mod_content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("#[path = \"")?;
            let path = rest.split('"').next()?;
            Some(path.to_string())
        })
        .collect()
}

fn rel_git_path(abs: &Path) -> String {
    abs.strip_prefix(manifest_root())
        .expect("under manifest root")
        .to_string_lossy()
        .replace('\\', "/")
}

pub fn collect_untracked_path_wired_modules() -> Vec<String> {
    let root = manifest_root();
    let mod_files = [
        root.join("src/lib.rs"),
        root.join("src/acp/mod.rs"),
        root.join("src/orchestrator/mod.rs"),
        root.join("src/coverage_kiss/mod.rs"),
        root.join("src/child_health/mod.rs"),
        root.join("src/acp_memory_containment/mod.rs"),
    ];
    let mut untracked = Vec::new();
    for mod_file in mod_files {
        let content = std::fs::read_to_string(&mod_file).expect("read mod file");
        for path_attr in path_attrs_in(&content) {
            let abs = rel_path_from_mod_file(&mod_file, &path_attr);
            let rel = rel_git_path(&abs);
            if !git_tracks(&rel) {
                untracked.push(rel);
            }
        }
    }
    untracked.sort();
    untracked.dedup();
    untracked
}

fn git_tracks(rel: &str) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", rel])
        .current_dir(manifest_root())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn lib_rs_top_level_module_names(lib_rs: &str) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    for line in lib_rs.lines() {
        let trimmed = line.trim();
        let rest = trimmed
            .strip_prefix("pub mod ")
            .or_else(|| trimmed.strip_prefix("pub(crate) mod "))
            .or_else(|| trimmed.strip_prefix("mod "));
        if let Some(rest) = rest {
            let name = rest
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
                .unwrap_or("");
            if !name.is_empty() {
                names.insert(name.to_string());
            }
        }
    }
    names
}

pub fn crate_top_modules_in_stringify_refs(
    stringify_refs: &str,
) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    for line in stringify_refs.lines() {
        let Some(rest) = line.split("stringify!(crate::").nth(1) else {
            continue;
        };
        let top = rest.split("::").next().unwrap_or("");
        if !top.is_empty() {
            names.insert(top.to_string());
        }
    }
    names
}

pub fn git_status_short_lines() -> Vec<String> {
    let out = Command::new("git")
        .args(["status", "--short"])
        .current_dir(manifest_root())
        .output()
        .expect("git status");
    assert!(out.status.success(), "git status failed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn collect_orchestrator_orphan_inc_paths(orchestrator_dir: &Path) -> Vec<String> {
    let mut orphans = Vec::new();
    for entry in std::fs::read_dir(orchestrator_dir).expect("read orchestrator dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) == Some("inc") {
            orphans.push(orphan_inc_note(&path));
        }
    }
    orphans
}

fn orphan_inc_note(path: &Path) -> String {
    let rs_path = path.with_extension("rs");
    let note = if rs_path.is_file() {
        let inc = std::fs::read_to_string(path).expect("read inc");
        let rs = std::fs::read_to_string(&rs_path).expect("read rs");
        if inc == rs {
            "duplicate of .rs, never include!d"
        } else {
            "drifted from matching .rs (stale/wrong code)"
        }
    } else {
        "no matching .rs"
    };
    format!("{} ({note})", path.display())
}
