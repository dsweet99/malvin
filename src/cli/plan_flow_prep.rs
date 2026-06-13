//! Preflight for `malvin plan`: open an existing `.md` file or materialize plan text on disk.

use std::path::PathBuf;

use crate::artifacts::{
    is_existing_md_file_path, looks_like_md_file_path_arg, resolve_user_md_request,
    write_plan_file_atomic,
};
use crate::cli::default_output_path::{
    allocate_default_sibling_file, DELIGHT_DEFAULT_OUT_PATH,
};

use super::PlanArgs;

pub(crate) fn resolve_plan_source_path(plan: &PlanArgs) -> Result<PathBuf, String> {
    let arg = plan.plan_path.trim();
    if let Some(path) = is_existing_md_file_path(arg) {
        return Ok(path);
    }
    if looks_like_md_file_path_arg(arg) {
        return Err(format!(
            "malvin plan: `{arg}` is not an existing .md file"
        ));
    }
    materialize_plan_text(arg, &plan.out_path)
}

fn materialize_plan_text(plan_text: &str, out_path: &str) -> Result<PathBuf, String> {
    let (text, _) = resolve_user_md_request(plan_text)?;
    if text.is_empty() {
        return Err("malvin plan: plan text must not be empty".into());
    }
    let resolved = resolve_plan_out_path(out_path)?;
    write_plan_file_atomic(&resolved, &text).map_err(|e| e.to_string())?;
    Ok(resolved)
}

fn resolve_plan_out_path(out_path: &str) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let resolved = if out_path == DELIGHT_DEFAULT_OUT_PATH {
        let default = cwd.join(DELIGHT_DEFAULT_OUT_PATH);
        allocate_default_sibling_file(&default, "plan", ".md")?
    } else {
        let resolved = cwd.join(out_path);
        if resolved.exists() {
            return Err(format!(
                "malvin plan: `{}` already exists; refusing to overwrite",
                resolved.display()
            ));
        }
        resolved
    };
    if let Some(parent) = resolved.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_args(plan_path: &str, out_path: &str) -> PlanArgs {
        PlanArgs {
            plan_path: plan_path.to_string(),
            out_path: out_path.to_string(),
        }
    }

    #[test]
    fn resolve_plan_source_path_requires_existing_md_for_path_mode() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "x").expect("write");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let got = resolve_plan_source_path(&plan_args("plan.md", "plan.md")).expect("resolve");
        std::env::set_current_dir(old).expect("restore");
        assert!(got.ends_with("plan.md"));
        assert!(resolve_plan_source_path(&plan_args("missing.md", "plan.md")).is_err());
    }

    #[test]
    fn resolve_plan_source_path_materializes_string_to_default_plan_md() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let cwd = std::env::current_dir().expect("cwd");
        let got = resolve_plan_source_path(&plan_args("Add caching layer", "plan.md")).expect("resolve");
        assert_eq!(got, cwd.join("plan.md"));
        assert_eq!(
            std::fs::read_to_string(&got).expect("read"),
            "Add caching layer"
        );
        std::env::set_current_dir(old).expect("restore");
    }

    #[test]
    fn resolve_plan_source_path_allocates_sibling_for_default_out_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("plan.md"), "occupied\n").expect("write");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let cwd = std::env::current_dir().expect("cwd");
        let got =
            resolve_plan_source_path(&plan_args("Ship widgets", "plan.md")).expect("resolve");
        assert_eq!(got, cwd.join("plan_1.md"));
        assert_eq!(
            std::fs::read_to_string(&got).expect("read"),
            "Ship widgets"
        );
        std::env::set_current_dir(old).expect("restore");
    }

    #[test]
    fn resolve_plan_source_path_writes_custom_out_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        let cwd = std::env::current_dir().expect("cwd");
        let got = resolve_plan_source_path(&plan_args(
            "Custom plan body",
            "plans/feature.md",
        ))
        .expect("resolve");
        assert_eq!(got, cwd.join("plans/feature.md"));
        assert_eq!(
            std::fs::read_to_string(&got).expect("read"),
            "Custom plan body"
        );
        std::env::set_current_dir(old).expect("restore");
    }

    #[test]
    fn resolve_plan_source_path_refuses_existing_custom_out_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("taken.md"), "x\n").expect("write");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        assert!(resolve_plan_source_path(&plan_args("body", "taken.md")).is_err());
        std::env::set_current_dir(old).expect("restore");
    }

    #[test]
    fn resolve_plan_source_path_rejects_empty_string_body() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let old = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");
        assert!(resolve_plan_source_path(&plan_args("   ", "plan.md")).is_err());
        std::env::set_current_dir(old).expect("restore");
    }
}
