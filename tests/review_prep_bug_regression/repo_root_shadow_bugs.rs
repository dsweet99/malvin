//! Bugs from `review_prep.md` § Bugs — untracked repo-root files shadow malvin run layout.

use super::helpers::{git_status_short_lines, manifest_root};

const SHADOW_ROOT_FILES: &[&str] = &["plan.md", ".malvin_checks"];

#[test]
fn manifest_ephemeral_quality_gates_must_not_persist_malvin_checks() {
    let root = manifest_root();
    let checks = root.join(".malvin_checks");
    if checks.exists() {
        return;
    }
    let md = malvin::repo_gates::prompt_quality_gates_markdown_ephemeral(&root)
        .expect("prompt_quality_gates_markdown_ephemeral");
    assert!(
        md.contains("kiss check"),
        "expected default gate markdown from ephemeral helper"
    );
    assert!(
        !checks.exists(),
        "bug: prompt_quality_gates_markdown_ephemeral must not leave repo-root .malvin_checks \
         when it was absent before the call"
    );
}

#[test]
fn repo_root_shadow_files_must_not_be_untracked() {
    let status = git_status_short_lines();
    let mut untracked_shadows = Vec::new();
    for name in SHADOW_ROOT_FILES {
        let path = manifest_root().join(name);
        if !path.exists() {
            continue;
        }
        let is_untracked = status.iter().any(|line| {
            (line.starts_with("?? ") || line.starts_with("??\t"))
                && line.split_whitespace().any(|part| part == *name)
        });
        if is_untracked {
            untracked_shadows.push(*name);
        }
    }
    assert!(
        untracked_shadows.is_empty(),
        "bug: untracked repo-root files shadow ./_malvin/<run>/ layout (track, gitignore, or \
         remove):\n{}",
        untracked_shadows.join("\n")
    );
}

fn read_manifest(rel: &str) -> String {
    std::fs::read_to_string(manifest_root().join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn prepare_tidy_run_must_backup_malvin_checks_before_workspace_gates() {
    let src = read_manifest("src/cli/tidy_flow/run_startup.inc");
    let backup = src
        .find("backup_workspace_malvin_checks_if_present")
        .expect("malvin_checks backup");
    let gates = src
        .find("run_repo_workspace_gates")
        .expect("workspace gates");
    assert!(
        backup < gates,
        "bug: prepare_tidy_run must snapshot .malvin_checks before gates materialize it"
    );
    let prepare = src
        .find("pub fn prepare_tidy_run")
        .expect("prepare_tidy_run");
    let body = &src[prepare..];
    assert!(
        !body.contains("ensure_default_malvin_checks_file"),
        "bug: prepare_tidy_run must not call ensure_default_malvin_checks_file (gates ensure \
         internally; early ensure pins backup to Present and leaves untracked .malvin_checks)"
    );
}

#[test]
fn plan_flow_must_backup_dotfiles_before_ensure_malvin_checks() {
    let src = read_manifest("src/cli/plan_flow/plan_flow_root.inc");
    let backup = src
        .find("backup_workspace_malvin_checks_if_present")
        .expect("malvin_checks backup");
    assert!(
        !src[backup..].contains("ensure_default_malvin_checks_file"),
        "bug: plan session startup must backup .malvin_checks before any ensure_default call"
    );
}

#[test]
fn kpop_prepare_must_snapshot_before_ephemeral_quality_gates_markdown() {
    let src = read_manifest("src/cli/kpop_flow_a.inc");
    let snapshot = src
        .find("SessionDotfileBackups::snapshot")
        .expect("dotfile snapshot");
    let ephemeral = src
        .find("prompt_quality_gates_markdown_ephemeral")
        .expect("ephemeral quality gates markdown");
    assert!(
        snapshot < ephemeral,
        "bug: kpop prepare must snapshot workspace dotfiles before prompt_quality_gates_markdown_ephemeral"
    );
}

fn fn_body<'a>(src: &'a str, fn_sig: &str) -> &'a str {
    let start = src
        .find(fn_sig)
        .unwrap_or_else(|| panic!("find {fn_sig}"));
    let rest = &src[start + fn_sig.len()..];
    let end = rest
        .find("\npub fn ")
        .or_else(|| rest.find("\npub(in "))
        .or_else(|| rest.find("\nmod "))
        .unwrap_or(rest.len());
    &src[start..start + fn_sig.len() + end]
}

#[test]
fn kpop_prepare_must_use_ephemeral_quality_gates_markdown() {
    let src = read_manifest("src/cli/kpop_flow_a.inc");
    let body = fn_body(&src, "pub(in crate::cli) fn prepare_kpop_run");
    assert!(
        body.contains("prompt_quality_gates_markdown_ephemeral"),
        "bug: prepare_kpop_run must use prompt_quality_gates_markdown_ephemeral so .malvin_checks \
         is not left untracked at repo root"
    );
    assert!(
        !body.contains("ensure_default_malvin_checks_file"),
        "bug: prepare_kpop_run must not call ensure_default_malvin_checks_file directly"
    );
}

#[test]
fn workflow_context_must_use_ephemeral_quality_gates_markdown() {
    let src = read_manifest("src/orchestrator/helpers.rs");
    let body = fn_body(&src, "pub fn workflow_context(");
    assert!(
        body.contains("prompt_quality_gates_markdown_ephemeral"),
        "bug: workflow_context must use prompt_quality_gates_markdown_ephemeral so .malvin_checks \
         is not left untracked at repo root after malvin code snapshots dotfiles"
    );
    assert!(
        !body.contains("ensure_default_malvin_checks_file"),
        "bug: workflow_context must not call ensure_default_malvin_checks_file directly"
    );
}

#[test]
fn code_flow_must_snapshot_dotfiles_before_workflow_context() {
    let src = read_manifest("src/cli/code_flow.rs");
    let snapshot = src
        .find("SessionDotfileBackups::snapshot")
        .expect("dotfile snapshot in run_code");
    let workflow = src
        .find("workflow_context(")
        .expect("workflow_context in run_code");
    assert!(
        snapshot < workflow,
        "bug: run_code must snapshot workspace dotfiles before workflow_context materializes \
         .malvin_checks"
    );
}
