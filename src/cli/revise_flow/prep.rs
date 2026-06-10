use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::prompts::{PromptError, PromptStore};
use crate::workflow_context::insert_formatted;

use super::super::{WorkflowCliOptions, prepare_kpop_prompt_store};
use crate::cli::workflow_kpop_shared::render_kpop_program_request;

pub(crate) fn prepare_revise_kpop_prompt_store(
    workflow: WorkflowCliOptions,
) -> Result<PromptStore, String> {
    let store = prepare_kpop_prompt_store(workflow, false)?;
    store
        .validate_exists("kpop_program.md")
        .map_err(|e: PromptError| e.0)?;
    store
        .validate_exists("revise_constraints.md")
        .map_err(|e: PromptError| e.0)?;
    Ok(store)
}

fn resolve_doc_path(doc_path: &str) -> Result<PathBuf, String> {
    let trimmed = doc_path.trim();
    if trimmed.is_empty() {
        return Err("malvin revise: missing required DOC_PATH".into());
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let path = Path::new(trimmed);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    if !resolved.is_file() {
        return Err(format!(
            "malvin revise: `{}` is not an existing file",
            resolved.display()
        ));
    }
    Ok(resolved)
}

pub(crate) fn revise_kpop_request(
    store: &PromptStore,
    artifacts: &crate::artifacts::RunArtifacts,
    resolved_doc_path: &Path,
) -> Result<String, String> {
    let workspace_root = artifacts.work_dir.as_path();
    let mut ctx = HashMap::new();
    insert_formatted(&mut ctx, "doc_path", resolved_doc_path, workspace_root);
    render_kpop_program_request(store, "revise_constraints.md", &ctx, artifacts)
}

pub(crate) fn revise_preflight(doc_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let trimmed = doc_path.trim();
    let resolved_doc_path = resolve_doc_path(trimmed)?;
    let work_dir = crate::artifacts::work_dir_for_path(Path::new(trimmed));
    Ok((resolved_doc_path, work_dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revise_kpop_request_has_no_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let doc = tmp.path().join("doc.md");
        std::fs::write(&doc, "# Doc\n\nSome text.\n").expect("write");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let artifacts =
            crate::artifacts::create_kpop_run_artifacts("revise", Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let text = revise_kpop_request(&store, &artifacts, &doc).expect("request");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(
            !text.contains("{{"),
            "revise kpop request must expand all placeholders: {text:?}"
        );
        assert!(
            text.contains("doc.md"),
            "expected doc_path in request: {text:?}"
        );
        assert!(
            text.contains("mystifying synonymy"),
            "expected revise_constraints in request: {text:?}"
        );
    }

    #[test]
    fn prepare_revise_kpop_prompt_store_loads_program_and_constraints() {
        let workflow = crate::cli::WorkflowCliOptions { force: false };
        let store = prepare_revise_kpop_prompt_store(workflow).expect("store");
        assert!(store.validate_exists("kpop_program.md").is_ok());
        assert!(store.validate_exists("revise_constraints.md").is_ok());
    }

    #[test]
    fn revise_preflight_rejects_missing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let err = revise_preflight("missing.md").expect_err("missing");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(err.contains("not an existing file"));
    }

    #[test]
    fn revise_preflight_accepts_existing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("doc.md"), "hello\n").expect("write");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let (resolved, work_dir) = revise_preflight("doc.md").expect("ok");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(resolved.ends_with("doc.md"));
        assert_eq!(work_dir, PathBuf::from("."));
    }

    #[test]
    fn revise_work_dir_is_parent_of_nested_doc_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("docs")).expect("mkdir");
        std::fs::write(tmp.path().join("docs/guide.md"), "hello\n").expect("write");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let (_resolved, work_dir) = revise_preflight("docs/guide.md").expect("ok");
        std::env::set_current_dir(old).expect("restore cwd");
        assert!(work_dir.ends_with("docs"));
    }
}
