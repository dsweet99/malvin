use crate::orchestrator::prompt_md_stem;
use crate::review_sync::{is_lgtm, sync_review_file};

fn tmp_review_paths() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    (t, workspace, artifact)
}

#[test]
fn prompt_md_stem_strips_suffix_without_panicking_on_short_names() {
    assert_eq!(prompt_md_stem("implement.md"), "implement");
    assert_eq!(prompt_md_stem("x.md"), "x");
    assert_eq!(prompt_md_stem(""), "");
    assert_eq!(prompt_md_stem("ab"), "ab");
    assert_eq!(prompt_md_stem("readme.markdown"), "readme.markdown");
}

#[test]
fn legacy_slice_stem_diverges_from_prompt_md_stem() {
    fn legacy_stem(s: &str) -> &str {
        &s[..s.len().saturating_sub(3)]
    }
    assert_eq!(legacy_stem("review_1.md"), prompt_md_stem("review_1.md"));
    assert_eq!(legacy_stem("review_2.md"), prompt_md_stem("review_2.md"));
    assert_ne!(
        legacy_stem("readme.markdown"),
        prompt_md_stem("readme.markdown")
    );
    assert_ne!(legacy_stem("review_1.MD"), prompt_md_stem("review_1.MD"));
}

#[test]
fn is_lgtm_reads_file() {
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("r.md");
    std::fs::write(&p, "LGTM\n").unwrap();
    assert!(is_lgtm(&p));
}

#[test]
fn sync_review_file_skips_empty_workspace_so_artifact_lgtm_is_preserved() {
    let (_t, workspace, artifact) = tmp_review_paths();
    std::fs::write(&workspace, "").unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    sync_review_file(&workspace, &artifact);
    assert_eq!(std::fs::read_to_string(&artifact).unwrap().trim(), "LGTM");
}

#[test]
fn sync_review_file_skips_whitespace_only_workspace() {
    let (_t, workspace, artifact) = tmp_review_paths();
    std::fs::write(&workspace, "  \n\t\n").unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    sync_review_file(&workspace, &artifact);
    assert_eq!(std::fs::read_to_string(&artifact).unwrap().trim(), "LGTM");
}

#[test]
fn sync_review_file_copies_nonempty_workspace_to_artifact() {
    let (_t, workspace, artifact) = tmp_review_paths();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    std::fs::write(&artifact, "old").unwrap();
    sync_review_file(&workspace, &artifact);
    assert_eq!(std::fs::read_to_string(&artifact).unwrap().trim(), "LGTM");
}
