// Startup log tag derived from the CLI `request` string.

use super::is_existing_md_file_path;

/// Label for the startup request log tag: `plan.md` → `plan`, `a/plan_1.md` → `plan_1`, else `prompt`.
#[must_use]
pub fn startup_request_tag_label(cli_request: &str) -> String {
    if let Some(path) = is_existing_md_file_path(cli_request) {
        return path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("plan")
            .to_string();
    }
    "prompt".to_string()
}

#[cfg(test)]
mod tests {
    use super::startup_request_tag_label;

    #[test]
    fn startup_request_tag_label_from_file_stem_or_prompt() {
        let _guard = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        crate::test_utils::with_cwd(tmp.path(), || {
            std::fs::write(tmp.path().join("plan.md"), "plan body\n").unwrap();
            assert_eq!(startup_request_tag_label("plan.md"), "plan");
            std::fs::create_dir_all(tmp.path().join("sub")).unwrap();
            std::fs::write(tmp.path().join("sub/plan_1.md"), "plan one\n").unwrap();
            assert_eq!(startup_request_tag_label("sub/plan_1.md"), "plan_1");
            assert_eq!(startup_request_tag_label("fix it"), "prompt");
        });
    }
}
