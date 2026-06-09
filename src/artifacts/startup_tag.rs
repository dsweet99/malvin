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
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        std::fs::write("plan.md", "x").unwrap();
        assert_eq!(startup_request_tag_label("plan.md"), "plan");
        std::fs::create_dir_all("sub").unwrap();
        std::fs::write("sub/plan_1.md", "y").unwrap();
        assert_eq!(startup_request_tag_label("sub/plan_1.md"), "plan_1");
        assert_eq!(startup_request_tag_label("fix it"), "prompt");
        std::env::set_current_dir(old_cwd).unwrap();
    }
}
