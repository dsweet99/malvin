//! Startup log tag derived from the CLI `request` string.

use std::path::Path;

/// Label for the startup request log tag: `@plan.md` → `plan`, `@a/plan_1.md` → `plan_1`, else `prompt`.
#[must_use]
pub fn startup_request_tag_label(cli_request: &str) -> String {
    let t = cli_request.trim();
    t.strip_prefix('@').map_or_else(
        || "prompt".to_string(),
        |path_str| {
            let path = Path::new(path_str);
            path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("plan")
                .to_string()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::startup_request_tag_label;

    #[test]
    fn startup_request_tag_label_from_file_stem_or_prompt() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("plan.md");
        std::fs::write(&p, "x").unwrap();
        assert_eq!(
            startup_request_tag_label(&format!("@{}", p.display())),
            "plan"
        );
        let p2 = tmp.path().join("sub").join("plan_1.md");
        std::fs::create_dir_all(p2.parent().unwrap()).unwrap();
        std::fs::write(&p2, "y").unwrap();
        assert_eq!(
            startup_request_tag_label(&format!("@{}", p2.display())),
            "plan_1"
        );
        assert_eq!(startup_request_tag_label("fix it"), "prompt");
    }
}
